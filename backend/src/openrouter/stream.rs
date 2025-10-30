use std::{pin::Pin, task};

use futures_util::FutureExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use tokio_stream::{Stream, StreamExt};

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
    toolcall: Option<ToolCall>,
    usage: Usage,
    stop_reason: Option<raw::FinishReason>,
    responses: Vec<StreamCompletionResp>,
    annotations: Option<Vec<serde_json::Value>>,
    model_id: Option<String>,
}

pub struct StreamResult {
    pub toolcall: Option<ToolCall>,
    pub usage: Usage,
    pub stop_reason: raw::FinishReason,
    pub responses: Vec<StreamCompletionResp>,
    pub annotations: Option<serde_json::Value>,
}

impl StreamCompletion {
    pub(super) async fn request(
        http_client: &Client,
        api_key: &str,
        endpoint: &str,
        req: raw::CompletionReq,
    ) -> Result<StreamCompletion, Error> {
        let model_id = req.model.clone();
        let builder = http_client
            .post(endpoint)
            .bearer_auth(api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req);

        match EventSource::new(builder) {
            Ok(source) => Ok(Self {
                source,
                toolcall: None,
                usage: Usage::default(),
                stop_reason: None,
                responses: vec![],
                annotations: None,
                model_id: Some(model_id),
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

        if let Some(reasoning) = delta.reasoning {
            return StreamCompletionResp::ReasoningToken(reasoning);
        }

        if let Some(call) = delta.tool_calls.map(|x| x.into_iter().next()).flatten() {
            if let Some(id) = call.id {
                self.toolcall = Some(ToolCall {
                    id,
                    ..Default::default()
                });
            }
            if let Some(state) = &mut self.toolcall {
                if let Some(name) = call.function.name {
                    state.name.push_str(&name);
                }
                if let Some(args) = call.function.arguments {
                    state.args.push_str(&args);
                }
            }
        }

        if let Some(reason) = choice.finish_reason {
            self.stop_reason = Some(reason.clone());
            return match reason {
                raw::FinishReason::Stop => StreamCompletionResp::ResponseToken(content),
                raw::FinishReason::Length => StreamCompletionResp::ResponseToken(content),
                raw::FinishReason::ToolCalls => match self.toolcall.clone() {
                    Some(call) => call.into(),
                    None => StreamCompletionResp::ResponseToken(content),
                },
            };
        }
        StreamCompletionResp::ResponseToken(content)
    }

    fn handle_data(&mut self, data: &str) -> Result<StreamCompletionResp, Error> {
        // this approach made it compatible with both openrouter and openai
        if let Ok(resp) = serde_json::from_str::<raw::CompletionInfoResp>(data) {
            self.usage.cost += resp.usage.cost;
            self.usage.token += resp.usage.total_tokens.unwrap_or(0);
            return Ok(StreamCompletionResp::Usage {
                price: resp.usage.cost,
                // cloak model may return null for total_tokens
                token: resp.usage.total_tokens.map(|x| x as usize).unwrap_or(0),
            });
        }

        let resp = serde_json::from_str::<raw::CompletionResp>(data)?;

        match (resp.model, &mut self.model_id) {
            (_, None) => {}
            (None, expect) => {
                log::warn!("Model ID not found in response");
                expect.take();
            }
            (Some(x), expect) => {
                if x != expect.as_ref().unwrap().as_str() {
                    log::warn!(
                        "Model ID mismatch: expected {}, got {}",
                        expect.take().unwrap(),
                        x
                    );
                }
            }
        };

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
                    message: format!(
                        "Openrouter return status code {}, message: {}",
                        code, error.error.message
                    ),
                    code: Some(code.as_u16() as i32),
                },
                Err(e) => Error::Api {
                    message: format!(
                        "Openrouter return status code {}, cannot parse error message: {}",
                        code, e
                    ),
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
        if self.stop_reason.is_none() {
            log::warn!("Provider didn't return finish_reason, set to Stop");
        }
        let stop_reason = self.stop_reason.take().unwrap_or(raw::FinishReason::Stop);

        StreamResult {
            toolcall: std::mem::take(&mut self.toolcall),
            usage: self.usage.clone(),
            stop_reason,
            responses: std::mem::take(&mut self.responses)
                .into_iter()
                .filter(|x| !x.is_empty())
                .collect(),
            annotations: self.annotations.take().map(serde_json::Value::Array),
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
    ToolCall {
        name: String,
        args: String,
        id: String,
    },
    ToolToken(String),
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

impl From<ToolCall> for StreamCompletionResp {
    fn from(value: ToolCall) -> Self {
        StreamCompletionResp::ToolCall {
            name: value.name,
            args: value.args,
            id: value.id,
        }
    }
}
