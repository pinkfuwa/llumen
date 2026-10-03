//! SSE streaming of chat completions.

use std::{collections::VecDeque, pin::Pin, task};

use futures_util::{FutureExt};
use http::header::CONTENT_TYPE;
use reqwest::{Body, Client};
use eventsource_stream::{Event, Eventsource, EventStreamError};
use stream_json::IntoSerializer;
use tokio_stream::{Stream, StreamExt};

use super::{error::Error, raw, GeneratedImage, OPENROUTER_HEADERS};

/// A single tool call accumulated from streaming delta chunks.
/// Fields are built incrementally from the stream and are final after
/// streaming completes.
#[derive(Default, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
}

/// Token usage and cost for a completion.
#[derive(Default, Clone)]
pub struct Usage {
    pub token: i64,
    pub cost: f64,
}

/// A streaming completion backed by an SSE event source.
/// Accumulates tool calls, usage, images, and citations as chunks arrive.
pub struct StreamCompletion {
    source: Pin<Box<dyn Stream<Item = Result<Event, EventStreamError<reqwest::Error>>> + Send>>,
    toolcalls: Vec<ToolCall>,
    usage: Usage,
    stop_reason: Option<raw::FinishReason>,
    responses: Vec<StreamCompletionResp>,
    annotations: Option<Vec<serde_json::Value>>,
    reasoning_details: Option<Vec<serde_json::Value>>,
    model_id: String,
    images: Vec<GeneratedImage>,
    citations: Vec<protocol::UrlCitation>,
    buffered: VecDeque<StreamCompletionResp>,
}

/// Final result collected from a completed `StreamCompletion`.
/// Contains all tool calls, usage, response tokens, and metadata.
pub struct StreamResult {
    pub toolcalls: Vec<ToolCall>,
    pub usage: Usage,
    pub stop_reason: raw::FinishReason,
    pub responses: Vec<StreamCompletionResp>,
    pub annotations: Option<serde_json::Value>,
    pub reasoning_details: Option<serde_json::Value>,
    pub image: Vec<GeneratedImage>,
    pub citations: Vec<protocol::UrlCitation>,
}

impl StreamResult {
    /// Returns the concatenated text of all `ResponseToken` entries,
    /// filtering out reasoning and tool tokens.
    pub fn get_text(&self) -> String {
        self.responses
            .iter()
            .filter_map(|t| match t {
                StreamCompletionResp::ResponseToken(token) => Some(token.clone()),
                _ => None,
            })
            .collect()
    }
}

impl StreamCompletion {
    pub(super) async fn request(
        http_client: &Client,
        api_key: &str,
        endpoint: &str,
        is_custom_api: bool,
        req: raw::CompletionReq,
    ) -> Result<StreamCompletion, Error> {
        let session_id = req.session_id.clone();
        let model_id = {
            let model_id = req.model.as_str();
            match model_id.find(":") {
                Some(pos) => model_id.split_at(pos).0,
                None => model_id,
            }
        }
        .to_string();

        let content_length = req.size();

        let body = Body::wrap_stream(req.into_stream());

        let mut builder = http_client
            .post(endpoint)
            .bearer_auth(api_key)
            .headers(OPENROUTER_HEADERS.clone())
            .header(CONTENT_TYPE, "application/json");

        if !is_custom_api {
            if let Some(ref sid) = session_id {
                builder = builder.header("x-session-id", sid);
            }
        }

        if let Some(len) = content_length {
            builder = builder.header(http::header::CONTENT_LENGTH, len);
        }
        let builder = builder.body(body);

        let response = builder.send().await?;
        if !response.status().is_success() {
            return Err(Self::handle_status_error(response).await);
        }
        let source = Box::pin(response.bytes_stream().eventsource());

        Ok(Self {
            source,
            toolcalls: Vec::new(),
            usage: Usage::default(),
            stop_reason: None,
            responses: vec![],
            annotations: None,
            reasoning_details: None,
            model_id,
            images: Vec::new(),
            citations: Vec::new(),
            buffered: VecDeque::new(),
        })
    }

    fn handle_choice(&mut self, choice: raw::Choice) -> Vec<StreamCompletionResp> {
        self.stop_reason = choice.finish_reason.or(self.stop_reason.take());
        let delta = choice.delta;

        let content = delta.content.unwrap_or("".to_string());

        if let Some(annotations) = delta.annotations {
            self.annotations
                .get_or_insert_with(|| Vec::with_capacity(1))
                .extend(annotations);
        }

        if let Some(reasoning_details) = delta.reasoning_details {
            self.reasoning_details
                .get_or_insert_with(|| Vec::with_capacity(1))
                .extend(reasoning_details);
        }

        // Extract citations from annotations
        if let Some(ref ann_array) = self.annotations {
            let ann_value = serde_json::Value::Array(ann_array.clone());
            let new_citations = super::annotation::extract_url_citations(&ann_value);
            for citation in new_citations {
                if !self.citations.iter().any(|c| c.url == citation.url) {
                    self.citations.push(citation);
                }
            }
        }

        // Handle images
        if !delta.images.is_empty() {
            for raw_image in delta.images {
                match GeneratedImage::from_raw_image(raw_image) {
                    Ok(image) => {
                        self.images.push(image);
                    }
                    Err(e) => {
                        log::error!("Failed to parse image: {}", e);
                    }
                }
            }
        }

        // Check for reasoning - emit reasoning BEFORE content when both exist
        let reasoning_token = if let Some(reasoning) = delta.reasoning {
            if !reasoning.is_empty() {
                Some(StreamCompletionResp::ReasoningToken(reasoning))
            } else {
                None
            }
        } else if let Some(reasoning) = delta.reasoning_content {
            if !reasoning.is_empty() {
                Some(StreamCompletionResp::ReasoningToken(reasoning))
            } else {
                None
            }
        } else {
            None
        };

        let mut responses = Vec::new();
        if let Some(reasoning) = reasoning_token {
            responses.push(reasoning);
        }
        if !content.is_empty() {
            responses.push(StreamCompletionResp::ResponseToken(content));
        }

        // Handle tool calls - support parallel tool calls
        if let Some(tool_calls) = delta.tool_calls {
            for call in tool_calls {
                let index = call.index as usize;

                // Ensure we have enough space for this tool call
                if self.toolcalls.len() <= index {
                    self.toolcalls.resize(index + 1, ToolCall::default());
                }

                // Initialize with id if present (first chunk for this tool call)
                if let Some(id) = call.id {
                    self.toolcalls[index].id = id;
                }

                let mut name_token = String::new();
                let mut args_token = String::new();

                // Accumulate tool name tokens
                if let Some(name) = call.function.name {
                    self.toolcalls[index].name.push_str(&name);
                    name_token = name;
                }

                // Accumulate tool arguments tokens
                if let Some(args) = call.function.arguments {
                    self.toolcalls[index].args.push_str(&args);
                    args_token = args;
                }

                if !name_token.is_empty() || !args_token.is_empty() {
                    responses.push(StreamCompletionResp::ToolToken {
                        idx: index,
                        name: name_token,
                        args: args_token,
                    });
                }
            }
        }

        responses
    }

    fn handle_data(&mut self, data: &str) -> Result<Vec<StreamCompletionResp>, Error> {
        // this approach made it compatible with both openrouter and openai
        let usage = if let Ok(resp) = serde_json::from_str::<raw::CompletionInfoResp>(data) {
            let cost = resp
                .usage
                .cost_details
                .map(|x| x.upstream_inference_cost)
                .flatten()
                .unwrap_or(resp.usage.cost);

            self.usage.cost += cost;
            self.usage.token += resp.usage.total_tokens.unwrap_or(0);
            Some(StreamCompletionResp::Usage {
                price: cost,
                // cloak model may return null for total_tokens
                token: resp.usage.total_tokens.map(|x| x as usize).unwrap_or(0),
            })
        } else {
            None
        };

        let resp = match serde_json::from_str::<raw::StreamCompletionResponse>(data) {
            Ok(response) => response,
            Err(_) if usage.is_some() => return Ok(usage.into_iter().collect()),
            Err(error) => return Err(error.into()),
        };

        if let Some(model_id) = resp.model {
            let trimmed_id = model_id.split(":").next().unwrap_or("");
            if !self.model_id.starts_with(trimmed_id) {
                log::warn!(
                    "Model ID mismatch: expected {}, got {}",
                    self.model_id,
                    model_id
                );
                self.model_id = model_id;
            }
        }

        if let Some(error) = resp.error {
            return Err(error.into());
        }

        let mut responses = match resp.choices.into_iter().next() {
            Some(choice) => self.handle_choice(choice),
            None if usage.is_some() => return Ok(usage.into_iter().collect()),
            None => return Err(Error::Incompatible("No returned choices in completion")),
        };
        self.responses.extend(responses.iter().cloned());
        responses.extend(usage);
        Ok(responses)
    }

    async fn handle_status_error(response: reqwest::Response) -> Error {
        let status = response.status();
        match response.json::<raw::ErrorResp>().await {
            Ok(error) => Error::Api {
                message: error.error.message,
                code: Some(status.as_u16() as i32),
            },
            Err(e) => Error::Api {
                message: format!("cannot parse error message: {}", e),
                code: Some(status.as_u16() as i32),
            },
        }
    }

    /// Advances the stream and returns the next completion chunk.
    /// Internally buffers items so that one SSE event may produce multiple
    /// chunks.
    pub async fn next(&mut self) -> Option<Result<StreamCompletionResp, Error>> {
        loop {
            // Return buffered items first
            if let Some(item) = self.buffered.pop_front() {
                return Some(Ok(item));
            }

            match self.source.next().await {
                Some(Ok(Event { data, .. })) if data != "[DONE]" => {
                    match self.handle_data(&data) {
                        Ok(items) => {
                            // Buffer all items except the first one
                            if items.is_empty() {
                                continue;
                            }
                            let first = items.first().cloned().unwrap();
                            for remaining in items.into_iter().skip(1) {
                                self.buffered.push_back(remaining);
                            }
                            return Some(Ok(first));
                        }
                        Err(Error::Incompatible(msg)) => {
                            log::warn!("Malbehave upstream: {}", msg);
                            continue;
                        }
                        Err(err) => return Some(Err(err)),
                    };
                }
                Some(Err(err)) => return Some(Err(err.into())),
                None => return None,
                _ => continue,
            }
        }
    }

    /// Consumes the stream and returns the final `StreamResult`.
    /// Determines the stop reason automatically: `ToolCalls` if any tool calls
    /// were accumulated, otherwise the provider-reported reason.
    pub fn get_result(mut self) -> StreamResult {
        let stop_reason = match self.toolcalls.is_empty() {
            true => self.stop_reason.clone().unwrap_or(raw::FinishReason::Stop),
            false => raw::FinishReason::ToolCalls,
        };

        if self.stop_reason.is_none() {
            log::warn!(
                "Provider didn't provide any finish reason, set to {:?}",
                stop_reason
            );
        } else if !self.toolcalls.is_empty()
            && matches!(self.stop_reason, Some(raw::FinishReason::Stop))
        {
            log::warn!("Provider returned stop reason when tool calls are present");
        }

        let reasoning_details = self.reasoning_details.take().map(|data| {
            serde_json::json!({
                "model_id": self.model_id.clone(),
                "data": data,
            })
        });

        StreamResult {
            toolcalls: std::mem::take(&mut self.toolcalls),
            usage: self.usage.clone(),
            stop_reason,
            responses: std::mem::take(&mut self.responses)
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect(),
            annotations: self.annotations.take().map(serde_json::Value::Array),
            reasoning_details,
            image: std::mem::take(&mut self.images),
            citations: std::mem::take(&mut self.citations),
        }
    }
}

// Please be aware that Stream implementation will skip empty string tokens

// For compatibility reason, we don't treat null and empty string differently
//
// And in openrouter's extension, they send null on special delta(annotation,
// tool call start, etc)
impl Stream for StreamCompletion {
    type Item = Result<StreamCompletionResp, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        let this = &mut *self;
        loop {
            let fut = StreamCompletion::next(this);
            let result = Box::pin(fut).poll_unpin(cx);
            if let task::Poll::Ready(Some(Ok(ref t))) = result {
                if StreamCompletionResp::is_empty(t) {
                    continue;
                }
            }
            return result;
        }
    }
}

/// A single chunk yielded during streaming.
/// Variants represent text tokens, reasoning tokens, tool-call tokens, and
/// usage info.
#[derive(Debug, Clone)]
pub enum StreamCompletionResp {
    ReasoningToken(String),
    ResponseToken(String),
    ToolToken {
        idx: usize,
        args: String,
        name: String,
    },
    Usage {
        price: f64,
        token: usize,
    },
}

impl StreamCompletionResp {
    /// Returns `true` if the chunk is a `ReasoningToken` or `ResponseToken`
    /// containing an empty string. Tool and usage tokens are never empty.
    pub fn is_empty(&self) -> bool {
        match self {
            StreamCompletionResp::ReasoningToken(s) => s.is_empty(),
            StreamCompletionResp::ResponseToken(s) => s.is_empty(),
            _ => false,
        }
    }
}

/// A wrapper that filters out tool tokens during streaming.
/// Tool calls are accumulated in the underlying StreamCompletion and available
/// via get_result() after streaming ends.
pub struct StreamWithOrderedTokens<S> {
    inner: S,
    stream_ended: bool,
}

impl<S> StreamWithOrderedTokens<S> {
    /// Creates a new wrapper that filters out tool tokens during streaming.
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            stream_ended: false,
        }
    }

    /// Consumes the wrapper and returns the inner stream.
    /// Should be called after streaming is complete.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S: Stream<Item = Result<StreamCompletionResp, Error>> + Unpin> Stream
    for StreamWithOrderedTokens<S>
{
    type Item = Result<StreamCompletionResp, Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        if self.stream_ended {
            return task::Poll::Ready(None);
        }

        match Pin::new(&mut self.inner).poll_next(cx) {
            task::Poll::Ready(Some(Ok(resp))) => {
                match resp {
                    StreamCompletionResp::ToolToken { .. } => {
                        // Skip tool tokens during streaming
                        // They are accumulated in StreamCompletion and available via get_result()
                        self.poll_next(cx)
                    }
                    other => task::Poll::Ready(Some(Ok(other))),
                }
            }
            task::Poll::Ready(Some(Err(e))) => task::Poll::Ready(Some(Err(e))),
            task::Poll::Ready(None) => {
                self.stream_ended = true;
                task::Poll::Ready(None)
            }
            task::Poll::Pending => task::Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn completion(events: Vec<Value>) -> StreamCompletion {
        let events = events.into_iter().map(|data| {
            Ok(Event {
                data: data.to_string(),
                ..Default::default()
            })
        });
        StreamCompletion {
            source: Box::pin(tokio_stream::iter(events)),
            toolcalls: Vec::new(),
            usage: Usage::default(),
            stop_reason: None,
            responses: Vec::new(),
            annotations: None,
            reasoning_details: None,
            model_id: "test/model".to_string(),
            images: Vec::new(),
            citations: Vec::new(),
            buffered: VecDeque::new(),
        }
    }

    fn chunk(delta: Value, finish_reason: Value) -> Value {
        json!({"id": "completion-test", "choices": [{
            "index": 0, "delta": delta, "finish_reason": finish_reason
        }]})
    }

    async fn drain(completion: &mut StreamCompletion) -> Result<Vec<StreamCompletionResp>, Error> {
        let mut responses = Vec::new();
        while let Some(response) = completion.next().await {
            responses.push(response?);
        }
        Ok(responses)
    }

    fn tokens(responses: &[StreamCompletionResp]) -> Vec<Value> {
        responses
            .iter()
            .map(|response| match response {
                StreamCompletionResp::ResponseToken(text) => json!(["text", text]),
                StreamCompletionResp::ReasoningToken(text) => json!(["reasoning", text]),
                StreamCompletionResp::ToolToken { idx, name, args } => {
                    json!(["tool", idx, name, args])
                }
                StreamCompletionResp::Usage { price, token } => json!(["usage", price, token]),
            })
            .collect()
    }

    #[tokio::test]
    async fn emits_reasoning_before_text_and_retains_finish_reason() -> Result<(), Error> {
        let mut stream = completion(vec![
            chunk(
                json!({"reasoning_content": "Think", "content": "Hello"}),
                json!(null),
            ),
            chunk(
                json!({"reasoning": "Again", "content": "世界"}),
                json!("length"),
            ),
        ]);
        assert_eq!(
            tokens(&drain(&mut stream).await?),
            vec![
                json!(["reasoning", "Think"]),
                json!(["text", "Hello"]),
                json!(["reasoning", "Again"]),
                json!(["text", "世界"]),
            ]
        );
        let result = stream.get_result();
        assert_eq!(result.get_text(), "Hello世界");
        assert!(matches!(result.stop_reason, raw::FinishReason::Length));
        Ok(())
    }

    #[tokio::test]
    async fn emits_every_parallel_tool_delta_and_accumulates_by_index() -> Result<(), Error> {
        let mut stream = completion(vec![
            chunk(
                json!({"reasoning": "Plan", "tool_calls": [
                    {"index": 0, "id": "first", "function": {"name": "sea", "arguments": "{\"q\":"}},
                    {"index": 1, "id": "second", "function": {"name": "crawl", "arguments": "{\"url\":"}}
                ]}),
                json!(null),
            ),
            chunk(
                json!({"tool_calls": [
                    {"index": 1, "function": {"arguments": "\"https://example.com\"}"}},
                    {"index": 0, "function": {"name": "rch", "arguments": "\"rust\"}"}}
                ]}),
                json!("stop"),
            ),
        ]);
        assert_eq!(
            tokens(&drain(&mut stream).await?),
            vec![
                json!(["reasoning", "Plan"]),
                json!(["tool", 0, "sea", "{\"q\":"]),
                json!(["tool", 1, "crawl", "{\"url\":"]),
                json!(["tool", 1, "", "\"https://example.com\"}"]),
                json!(["tool", 0, "rch", "\"rust\"}"]),
            ]
        );
        let result = stream.get_result();
        assert_eq!(result.toolcalls.len(), 2);
        assert_eq!(result.toolcalls[0].id, "first");
        assert_eq!(result.toolcalls[0].name, "search");
        assert_eq!(result.toolcalls[0].args, r#"{"q":"rust"}"#);
        assert_eq!(result.toolcalls[1].id, "second");
        assert_eq!(result.toolcalls[1].name, "crawl");
        assert_eq!(result.toolcalls[1].args, r#"{"url":"https://example.com"}"#);
        assert!(matches!(result.stop_reason, raw::FinishReason::ToolCalls));
        Ok(())
    }

    #[tokio::test]
    async fn retains_content_in_a_tool_call_chunk() -> Result<(), Error> {
        let mut stream = completion(vec![chunk(
            json!({
                "content": "Searching", "tool_calls": [{
                    "index": 0, "id": "call", "function": {"name": "search", "arguments": "{}"}
                }]
            }),
            json!("tool_calls"),
        )]);
        let responses = drain(&mut stream).await?;
        assert_eq!(
            tokens(&responses),
            vec![
                json!(["text", "Searching"]),
                json!(["tool", 0, "search", "{}"])
            ]
        );
        assert_eq!(stream.get_result().get_text(), "Searching");
        Ok(())
    }

    #[tokio::test]
    async fn processes_content_and_usage_from_the_same_event() -> Result<(), Error> {
        let mut event = chunk(json!({"content": "Answer"}), json!("stop"));
        event["model"] = json!("test/model");
        event["usage"] = json!({"total_tokens": 7, "cost": 0.25});
        let mut stream = completion(vec![event]);
        assert_eq!(
            tokens(&drain(&mut stream).await?),
            vec![json!(["text", "Answer"]), json!(["usage", 0.25, 7])]
        );
        let result = stream.get_result();
        assert_eq!(result.get_text(), "Answer");
        assert_eq!(result.usage.token, 7);
        assert_eq!(result.usage.cost, 0.25);
        Ok(())
    }

    #[tokio::test]
    async fn handles_usage_only_events_and_nullable_token_counts() -> Result<(), Error> {
        let mut stream = completion(vec![
            json!({"id": "test", "model": "test/model", "usage": {
                "total_tokens": 7, "cost": 1.0, "cost_details": {"upstream_inference_cost": 0.25}
            }}),
            json!({"id": "test", "model": "test/model", "usage": {"total_tokens": null, "cost": 0.5}}),
        ]);
        assert_eq!(
            tokens(&drain(&mut stream).await?),
            vec![json!(["usage", 0.25, 7]), json!(["usage", 0.5, 0])]
        );
        let result = stream.get_result();
        assert_eq!(result.usage.token, 7);
        assert_eq!(result.usage.cost, 0.75);
        assert!(result.responses.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn preserves_metadata_and_deduplicates_citations_across_events() -> Result<(), Error> {
        let citation =
            json!({"type": "url_citation", "url_citation": {"url": "https://example.com"}});
        let mut stream = completion(vec![
            chunk(
                json!({"annotations": [citation.clone()], "reasoning_details": [{"text": "first"}]}),
                json!(null),
            ),
            chunk(
                json!({"annotations": [citation], "reasoning_details": [{"text": "second"}]}),
                json!(null),
            ),
        ]);
        assert!(drain(&mut stream).await?.is_empty());
        let result = stream.get_result();
        assert_eq!(result.citations.len(), 1);
        assert_eq!(result.citations[0].url, "https://example.com");
        assert_eq!(
            result
                .annotations
                .as_ref()
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            result.reasoning_details,
            Some(json!({
                "model_id": "test/model", "data": [{"text": "first"}, {"text": "second"}]
            }))
        );
        Ok(())
    }

    #[tokio::test]
    async fn skips_empty_choices_but_reports_upstream_errors() -> Result<(), Error> {
        let mut stream = completion(vec![
            json!({"id": "test", "choices": []}),
            chunk(json!({"content": "Before error"}), json!(null)),
            json!({"id": "test", "choices": [], "error": {"message": "Quota exceeded", "code": 429}}),
        ]);
        let first = stream.next().await.transpose()?;
        assert!(
            matches!(first, Some(StreamCompletionResp::ResponseToken(ref text)) if text == "Before error")
        );
        assert!(
            matches!(stream.next().await, Some(Err(Error::Api { message, code: Some(429) })) if message == "Quota exceeded")
        );
        Ok(())
    }

    #[test]
    fn reports_malformed_json() {
        let mut stream = completion(Vec::new());
        assert!(matches!(
            stream.handle_data("not json"),
            Err(Error::Serde(_))
        ));
    }

    #[tokio::test]
    async fn stream_trait_filters_empty_tokens_and_keeps_buffered_order() -> Result<(), Error> {
        let mut stream = completion(vec![
            chunk(json!({"content": ""}), json!(null)),
            chunk(
                json!({"reasoning": "Think", "content": "Answer"}),
                json!(null),
            ),
            chunk(json!({}), json!("stop")),
        ]);
        let mut responses = Vec::new();
        while let Some(response) = StreamExt::next(&mut stream).await {
            responses.push(response?);
        }
        assert_eq!(
            tokens(&responses),
            vec![json!(["reasoning", "Think"]), json!(["text", "Answer"])]
        );
        assert_eq!(stream.get_result().responses.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn retains_each_reported_finish_reason() -> Result<(), Error> {
        for reason in ["stop", "length", "error", "tool_calls"] {
            let mut stream = completion(vec![chunk(json!({"content": "Answer"}), json!(reason))]);
            drain(&mut stream).await?;
            let result = stream.get_result();
            let expected: raw::FinishReason = serde_json::from_value(json!(reason))?;
            assert_eq!(
                std::mem::discriminant(&result.stop_reason),
                std::mem::discriminant(&expected)
            );
            assert_eq!(result.get_text(), "Answer");
        }
        Ok(())
    }

    #[tokio::test]
    async fn skips_done_marker_and_defaults_missing_finish_reason() -> Result<(), Error> {
        let mut stream = completion(Vec::new());
        stream.source = Box::pin(tokio_stream::iter(vec![
            Ok(Event {
                data: chunk(json!({"content": "Answer"}), json!(null)).to_string(),
                ..Default::default()
            }),
            Ok(Event {
                data: "[DONE]".to_string(),
                ..Default::default()
            }),
        ]));
        assert_eq!(
            tokens(&drain(&mut stream).await?),
            vec![json!(["text", "Answer"])]
        );
        assert!(matches!(
            stream.get_result().stop_reason,
            raw::FinishReason::Stop
        ));
        Ok(())
    }

    #[tokio::test]
    async fn ordered_wrapper_filters_tools_and_propagates_errors() {
        let source = tokio_stream::iter(vec![
            Ok(StreamCompletionResp::ToolToken {
                idx: 0,
                name: "search".into(),
                args: "{}".into(),
            }),
            Ok(StreamCompletionResp::ResponseToken("Answer".into())),
            Err(Error::Incompatible("test error")),
        ]);
        let mut stream = StreamWithOrderedTokens::new(source);
        assert!(
            matches!(StreamExt::next(&mut stream).await, Some(Ok(StreamCompletionResp::ResponseToken(text))) if text == "Answer")
        );
        assert!(matches!(
            StreamExt::next(&mut stream).await,
            Some(Err(Error::Incompatible("test error")))
        ));
        assert!(StreamExt::next(&mut stream).await.is_none());
        assert!(StreamExt::next(&mut stream).await.is_none());
    }
}
