use std::{pin::Pin, task};

use bytes::Bytes;
use futures_util::{FutureExt, StreamExt as FuturesStreamExt};
use reqwest::Client;
use tokio_stream::Stream;

use super::Image;
use super::stream_encode::{MessageWithStreams, serialize_to_body};
use super::{HTTP_REFERER, SyncStream, X_TITLE, error::Error, raw};

#[derive(Default, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
}

#[derive(Default, Clone)]
pub struct Usage {
    pub token: i64,
    pub cost: f64,
}

pub struct StreamCompletion {
    source: Pin<Box<dyn Stream<Item = Result<eventsource_stream::Event, std::io::Error>> + Send>>,
    toolcalls: Vec<ToolCall>,
    usage: Usage,
    stop_reason: Option<raw::FinishReason>,
    responses: Vec<StreamCompletionResp>,
    annotations: Option<Vec<serde_json::Value>>,
    reasoning_details: Option<Vec<serde_json::Value>>,
    model_id: String,
    images: Vec<Image>,
}

pub struct StreamResult {
    pub toolcalls: Vec<ToolCall>,
    pub usage: Usage,
    pub stop_reason: raw::FinishReason,
    pub responses: Vec<StreamCompletionResp>,
    pub annotations: Option<serde_json::Value>,
    pub reasoning_details: Option<serde_json::Value>,
    pub image: Vec<Image>,
}

impl StreamResult {
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
        req: raw::CompletionReq,
    ) -> Result<StreamCompletion, Error> {
        let model_id = {
            let model_id = req.model.as_str();
            match model_id.find(":") {
                Some(pos) => model_id.split_at(pos).0,
                None => model_id,
            }
        }
        .to_string();

        let request = http_client
            .post(endpoint)
            .bearer_auth(api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req)
            .build()
            .map_err(|e| Error::Api {
                message: format!("Failed to build request: {}", e),
                code: None,
            })?;

        // Send request and get response stream
        let response = http_client.execute(request).await.map_err(|e| Error::Api {
            message: format!("Failed to execute request: {}", e),
            code: None,
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                message: format!("Request failed with status {}: {}", status, error_body),
                code: Some(status.as_u16() as i32),
            });
        }

        // Create manual SSE parser
        let byte_stream = response.bytes_stream();
        let sse_stream = parse_sse_stream(byte_stream);

        Ok(Self {
            source: Box::pin(sse_stream),
            toolcalls: Vec::new(),
            usage: Usage::default(),
            stop_reason: None,
            responses: vec![],
            annotations: None,
            reasoning_details: None,
            model_id,
            images: Vec::new(),
        })
    }

    pub(super) async fn request_streaming<S: SyncStream + Send + 'static>(
        http_client: &Client,
        api_key: &str,
        endpoint: &str,
        req: raw::CompletionReq,
        messages_with_streams: Vec<MessageWithStreams<S>>,
    ) -> Result<StreamCompletion, Error> {
        let model_id = {
            let model_id = req.model.as_str();
            match model_id.find(":") {
                Some(pos) => model_id.split_at(pos).0,
                None => model_id,
            }
        }
        .to_string();

        // Serialize to streaming body
        let (rx, handle) = serialize_to_body(req, messages_with_streams).await;

        let body_stream = FuturesStreamExt::filter_map(
            tokio_stream::wrappers::ReceiverStream::new(rx),
            |result| async move {
                match result {
                    Ok(bytes) => Some(Ok::<Bytes, String>(bytes)),
                    Err(e) => {
                        log::error!("Streaming body error: {}", e);
                        None
                    }
                }
            },
        );

        let body = reqwest::Body::wrap_stream(body_stream);

        let request = http_client
            .post(endpoint)
            .bearer_auth(api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .header("Content-Type", "application/json")
            .body(body)
            .build()
            .map_err(|e| Error::Api {
                message: format!("Failed to build request: {}", e),
                code: None,
            })?;

        // Send request and get response stream
        let response = http_client.execute(request).await.map_err(|e| Error::Api {
            message: format!("Failed to execute request: {}", e),
            code: None,
        })?;

        // Check status
        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Api {
                message: format!("Request failed with status {}: {}", status, error_body),
                code: Some(status.as_u16() as i32),
            });
        }

        // Create manual SSE parser
        let byte_stream = response.bytes_stream();
        let sse_stream = parse_sse_stream(byte_stream);

        // Start serialization task in background
        tokio::spawn(async move {
            if let Err(e) = handle.await {
                log::error!("Serialization task failed: {}", e);
            }
        });

        Ok(Self {
            source: Box::pin(sse_stream),
            toolcalls: Vec::new(),
            usage: Usage::default(),
            stop_reason: None,
            responses: vec![],
            annotations: None,
            reasoning_details: None,
            model_id,
            images: Vec::new(),
        })
    }

    fn handle_choice(&mut self, choice: raw::Choice) -> StreamCompletionResp {
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

        // Handle images
        if !delta.images.is_empty() {
            for raw_image in delta.images {
                match Image::from_raw_image(raw_image) {
                    Ok(image) => {
                        self.images.push(image);
                    }
                    Err(e) => {
                        log::error!("Failed to parse image: {}", e);
                    }
                }
            }
        }

        if let Some(reasoning) = delta.reasoning {
            if !reasoning.is_empty() {
                return StreamCompletionResp::ReasoningToken(reasoning);
            }
        } else if let Some(reasoning) = delta.reasoning_content {
            if !reasoning.is_empty() {
                return StreamCompletionResp::ReasoningToken(reasoning);
            }
        }

        // Handle tool calls - support parallel tool calls
        if let Some(tool_calls) = delta.tool_calls {
            let mut last_tool_token: Option<(usize, String, String)> = None;

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

                // Track the last non-empty token
                if !name_token.is_empty() || !args_token.is_empty() {
                    last_tool_token = Some((index, name_token, args_token));
                }
            }

            if let Some((idx, name, args)) = last_tool_token {
                return StreamCompletionResp::ToolToken { idx, name, args };
            }
        }

        if let Some(reason) = choice.finish_reason {
            self.stop_reason = Some(reason.clone());
            return match reason {
                raw::FinishReason::Stop | raw::FinishReason::Length | raw::FinishReason::Error => {
                    StreamCompletionResp::ResponseToken(content)
                }
                raw::FinishReason::ToolCalls => {
                    // Return first tool call when finish_reason is ToolCalls
                    // The full list is available in get_result()

                    StreamCompletionResp::ResponseToken(content)
                }
            };
        }
        StreamCompletionResp::ResponseToken(content)
    }

    fn handle_data(&mut self, data: &str) -> Result<StreamCompletionResp, Error> {
        // this approach made it compatible with both openrouter and openai
        if let Ok(resp) = serde_json::from_str::<raw::CompletionInfoResp>(data) {
            let cost = resp
                .usage
                .cost_details
                .map(|x| x.upstream_inference_cost)
                .flatten()
                .unwrap_or(resp.usage.cost);

            self.usage.cost += cost;
            self.usage.token += resp.usage.total_tokens.unwrap_or(0);
            return Ok(StreamCompletionResp::Usage {
                price: cost,
                // cloak model may return null for total_tokens
                token: resp.usage.total_tokens.map(|x| x as usize).unwrap_or(0),
            });
        }

        let resp = serde_json::from_str::<raw::StreamCompletionResponse>(data)?;

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

        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or(Error::Incompatible("No returned choices in completion"))?;

        let resp = self.handle_choice(choice);
        self.responses.push(resp.clone());
        Ok(resp)
    }

    pub async fn next(&mut self) -> Option<Result<StreamCompletionResp, Error>> {
        use futures_util::StreamExt;
        loop {
            match self.source.next().await? {
                Ok(event) if &event.data != "[DONE]" => {
                    return match self.handle_data(&event.data) {
                        Ok(x) => Some(Ok(x)),
                        Err(Error::Incompatible(msg)) => {
                            log::warn!("Malbehave upstream: {}", msg);
                            continue;
                        }
                        Err(err) => Some(Err(err)),
                    };
                }
                Err(e) => {
                    log::error!("Stream error: {}", e);
                    return Some(Err(Error::Io(e)));
                }
                _ => continue,
            }
        }
    }

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
        }
    }
}

// Please be aware that Stream implementation will skip empty string tokens

// For compatibility reason, we don't treat null and empty string differently
//
// And in openrouter's extension, they send null on special delta(annotation, tool call start, etc)
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

// Parse SSE stream from raw bytes
fn parse_sse_stream(
    byte_stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<eventsource_stream::Event, std::io::Error>> + Send {
    SseParser::new(byte_stream)
}

struct SseParser {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    buffer: String,
}

impl SseParser {
    fn new(inner: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(inner),
            buffer: String::new(),
        }
    }
}

impl Stream for SseParser {
    type Item = Result<eventsource_stream::Event, std::io::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        loop {
            // First, try to parse an event from the buffer
            if let Some(pos) = self.buffer.find("\n\n") {
                let message = self.buffer[..pos].to_string();
                self.buffer.drain(..pos + 2);

                // Parse SSE message - handle multiple lines of data
                let mut data_lines = Vec::new();
                for line in message.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        data_lines.push(data);
                    } else if line.starts_with("data:") {
                        data_lines.push(&line[5..]);
                    }
                }

                if !data_lines.is_empty() {
                    let data = data_lines.join("\n");
                    return task::Poll::Ready(Some(Ok(eventsource_stream::Event {
                        event: "message".to_string(),
                        data,
                        id: String::new(),
                        retry: None,
                    })));
                }
                // If no data lines, continue to parse next message
                continue;
            }

            // No complete message in buffer, read more bytes
            match self.inner.as_mut().poll_next(cx) {
                task::Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                    // Loop back to try parsing again
                }
                task::Poll::Ready(Some(Err(e))) => {
                    return task::Poll::Ready(Some(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))));
                }
                task::Poll::Ready(None) => {
                    return task::Poll::Ready(None);
                }
                task::Poll::Pending => {
                    return task::Poll::Pending;
                }
            }
        }
    }
}

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
    pub fn is_empty(&self) -> bool {
        match self {
            StreamCompletionResp::ReasoningToken(s) => s.is_empty(),
            StreamCompletionResp::ResponseToken(s) => s.is_empty(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_sse_parser_basic() {
        let data = b"data: hello\n\ndata: world\n\n".to_vec();
        let stream =
            futures_util::stream::once(async { Ok::<Bytes, reqwest::Error>(Bytes::from(data)) });

        let mut parser = parse_sse_stream(stream);

        let event1 = parser.next().await.unwrap().unwrap();
        assert_eq!(event1.data, "hello");

        let event2 = parser.next().await.unwrap().unwrap();
        assert_eq!(event2.data, "world");

        assert!(parser.next().await.is_none());
    }

    #[tokio::test]
    async fn test_sse_parser_multiline_data() {
        let data = b"data: line1\ndata: line2\n\n".to_vec();
        let stream =
            futures_util::stream::once(async { Ok::<Bytes, reqwest::Error>(Bytes::from(data)) });

        let mut parser = parse_sse_stream(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "line1\nline2");

        assert!(parser.next().await.is_none());
    }

    #[tokio::test]
    async fn test_sse_parser_chunked_input() {
        // Simulate SSE data arriving in chunks
        let chunks = vec![
            Ok::<Bytes, reqwest::Error>(Bytes::from("data: hel")),
            Ok(Bytes::from("lo\n\nda")),
            Ok(Bytes::from("ta: world\n\n")),
        ];
        let stream = futures_util::stream::iter(chunks);

        let mut parser = parse_sse_stream(stream);

        let event1 = parser.next().await.unwrap().unwrap();
        assert_eq!(event1.data, "hello");

        let event2 = parser.next().await.unwrap().unwrap();
        assert_eq!(event2.data, "world");

        assert!(parser.next().await.is_none());
    }

    #[tokio::test]
    async fn test_sse_parser_data_without_space() {
        let data = b"data:hello\n\n".to_vec();
        let stream =
            futures_util::stream::once(async { Ok::<Bytes, reqwest::Error>(Bytes::from(data)) });

        let mut parser = parse_sse_stream(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "hello");

        assert!(parser.next().await.is_none());
    }

    #[tokio::test]
    async fn test_sse_parser_empty_lines() {
        let data = b"data: test\n\n\n\ndata: next\n\n".to_vec();
        let stream =
            futures_util::stream::once(async { Ok::<Bytes, reqwest::Error>(Bytes::from(data)) });

        let mut parser = parse_sse_stream(stream);

        let event1 = parser.next().await.unwrap().unwrap();
        assert_eq!(event1.data, "test");

        let event2 = parser.next().await.unwrap().unwrap();
        assert_eq!(event2.data, "next");

        assert!(parser.next().await.is_none());
    }
}
