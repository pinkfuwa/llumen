use std::{pin::Pin, task};

use futures_util::FutureExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use tokio_stream::{Stream, StreamExt};

use super::Image;

use super::{HTTP_REFERER, X_TITLE, error::Error, raw};

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
    source: EventSource,
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

        let builder = http_client
            .post(endpoint)
            .bearer_auth(api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req);

        match EventSource::new(builder) {
            Ok(source) => Ok(Self {
                source,
                toolcalls: Vec::new(),
                usage: Usage::default(),
                stop_reason: None,
                responses: vec![],
                annotations: None,
                reasoning_details: None,
                model_id,
                images: Vec::new(),
            }),
            Err(e) => {
                log::error!("Failed to create event source: {}", e);
                Err(Error::CannotCloneRequest(e))
            }
        }
    }

    pub fn close(&mut self) {
        self.source.close();
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
            return StreamCompletionResp::ReasoningToken(reasoning);
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

    async fn handle_error(&self, err: reqwest_eventsource::Error) -> Error {
        use reqwest_eventsource::Error as EventErr;
        if let EventErr::InvalidStatusCode(code, res) = err {
            match res.json::<raw::ErrorResp>().await {
                Ok(error) => Error::Api {
                    message: error.error.message,
                    code: Some(code.as_u16() as i32),
                },
                Err(e) => Error::Api {
                    message: format!("cannot parse error message: {}", e),
                    code: Some(code.as_u16() as i32),
                },
            }
        } else {
            Error::EventSource(err)
        }
    }

    pub async fn next(&mut self) -> Option<Result<StreamCompletionResp, Error>> {
        loop {
            match self.source.next().await? {
                Ok(Event::Message(e)) if &e.data != "[DONE]" => {
                    return match self.handle_data(&e.data) {
                        Ok(x) => Some(Ok(x)),
                        Err(Error::Incompatible(msg)) => {
                            log::warn!("Malbehave upstream: {}", msg);
                            continue;
                        }
                        Err(err) => Some(Err(err)),
                    };
                }
                Err(e) => {
                    return match e {
                        reqwest_eventsource::Error::StreamEnded => {
                            log::debug!("Stream ended");
                            None
                        }
                        e => Some(Err(self.handle_error(e).await)),
                    };
                }
                _ => continue,
            }
        }
    }

    pub fn get_result(mut self) -> StreamResult {
        let stop_reason = self.stop_reason.take().unwrap_or_else(|| {
            log::warn!("Provider didn't return finish_reason");
            match self.toolcalls.is_empty() {
                true => raw::FinishReason::Stop,
                false => raw::FinishReason::ToolCalls,
            }
        });

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

impl Drop for StreamCompletion {
    fn drop(&mut self) {
        self.source.close();
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
