use std::{pin::Pin, task};

use anyhow::{Context, Result, anyhow};
use futures_util::{FutureExt, Stream, StreamExt};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};

use super::{HTTP_REFERER, X_TITLE, raw};

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
}

pub struct StreamResult {
    pub toolcall: Option<ToolCall>,
    pub usage: Usage,
    pub stop_reason: raw::FinishReason,
    pub responses: Vec<StreamCompletionResp>,
}

impl StreamCompletion {
    pub(super) async fn request(
        http_client: &Client,
        api_key: &str,
        endpoint: &str,
        req: raw::CompletionReq,
    ) -> Result<StreamCompletion> {
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
            }),
            Err(e) => {
                tracing::error!("Failed to create event source: {}", e);
                Err(anyhow!("Failed to create event source: {}", e))
            }
        }
    }

    pub fn close(&mut self) {
        self.source.close();
    }

    fn handle_choice(&mut self, choice: raw::Choice) -> StreamCompletionResp {
        let delta = choice.delta;

        let content = delta.content.unwrap_or("".to_string());

        if let Some(reasoning) = delta.reasoning {
            return StreamCompletionResp::ResponseToken(reasoning);
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
        return StreamCompletionResp::ResponseToken(content);
    }

    fn handle_data(&mut self, data: &str) -> Result<StreamCompletionResp> {
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

        let resp = serde_json::from_str::<raw::CompletionResp>(data).context("Parse error")?;

        let choice = resp
            .choices
            .into_iter()
            .next()
            .ok_or(anyhow!("No returned choices in completion"))?;

        let resp = self.handle_choice(choice);
        self.responses.push(resp.clone());
        Ok(resp)
    }

    async fn handle_error(&self, err: reqwest_eventsource::Error) -> anyhow::Error {
        if let reqwest_eventsource::Error::InvalidStatusCode(code, res) = err {
            return match res
                .json::<raw::ErrorResp>()
                .await
                .context("Stream Error, cannot capture error message")
            {
                Ok(error) => {
                    anyhow!(
                        "Openrouter return status code {}, message: {}",
                        code,
                        error.error.message
                    )
                }
                Err(x) => anyhow!(
                    "Openrouter return status code {}, cannot parse error message: {}",
                    code,
                    x
                ),
            };
        } else {
            tracing::error!("Stream error: {}", err);

            return err.into();
        }
    }

    pub async fn next(&mut self) -> Option<Result<StreamCompletionResp>> {
        loop {
            match self.source.next().await? {
                Ok(Event::Open) => continue,
                Ok(Event::Message(e)) if &e.data != "[DONE]" => {
                    return Some(self.handle_data(&e.data));
                }
                Err(e) => {
                    return match e {
                        reqwest_eventsource::Error::StreamEnded => {
                            tracing::debug!("Stream ended");
                            None
                        }
                        e => Some(Err(self.handle_error(e).await)),
                    };
                }
                _ => return None,
            }
        }
    }

    pub fn get_result(self) -> Result<StreamResult> {
        Ok(StreamResult {
            toolcall: self.toolcall.clone(),
            usage: self.usage.clone(),
            stop_reason: self.stop_reason.clone().context("Stream not finished")?,
            responses: self.responses.clone(),
        })
    }
}

impl Stream for StreamCompletion {
    type Item = Result<StreamCompletionResp>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        let fut = self.next();
        Box::pin(fut).poll_unpin(cx)
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

impl From<ToolCall> for StreamCompletionResp {
    fn from(value: ToolCall) -> Self {
        StreamCompletionResp::ToolCall {
            name: value.name,
            args: value.args,
            id: value.id,
        }
    }
}
