use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};

use super::{HTTP_REFERER, X_TITLE, raw};

pub struct StreamCompletion {
    source: EventSource,
}

impl StreamCompletion {
    pub(super) async fn request(
        api_key: &str,
        endpoint: &str,
        req: raw::CompletionReq,
    ) -> Result<StreamCompletion> {
        let builder = Client::new()
            .post(endpoint)
            .bearer_auth(api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req);

        match EventSource::new(builder) {
            Ok(source) => Ok(Self { source }),
            Err(e) => {
                tracing::error!(target: "stream_completion", "Failed to create event source: {}", e);
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
            return StreamCompletionResp::ReasoningToken(reasoning);
        }

        tracing::trace!(target: "stream_completion", "Received token: {}", content);
        if let Some(reason) = choice.finish_reason {
            return match reason {
                raw::FinishReason::Stop => StreamCompletionResp::ResponseToken(content),
                raw::FinishReason::Length => StreamCompletionResp::ResponseToken(content),
                raw::FinishReason::ToolCalls => {
                    let tool_calls = delta.tool_calls.map(|x| x.into_iter().next()).flatten();
                    match tool_calls {
                        Some(tool_call) => StreamCompletionResp::ToolCall {
                            name: tool_call.function.name,
                            args: tool_call.function.arguments,
                            id: tool_call.id,
                        },
                        None => StreamCompletionResp::ResponseToken(content),
                    }
                }
            };
        }
        return StreamCompletionResp::ResponseToken(content);
    }

    fn handle_data(&mut self, data: &str) -> Result<StreamCompletionResp> {
        // this approach made it compatible with both openrouter and openai
        if let Ok(resp) = serde_json::from_str::<raw::CompletionInfoResp>(data) {
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

        Ok(self.handle_choice(choice))
    }

    pub async fn next(&mut self) -> Option<Result<StreamCompletionResp>> {
        loop {
            match self.source.next().await? {
                Ok(Event::Open) => continue,
                Ok(Event::Message(e)) if &e.data != "[DONE]" => {
                    return Some(self.handle_data(&e.data));
                }
                Err(e) => match e {
                    reqwest_eventsource::Error::StreamEnded => {
                        tracing::debug!(target: "stream_completion", "Stream ended");
                        return None;
                    }
                    e => {
                        tracing::error!(target: "stream_completion", "Stream error: {}", e);
                        return Some(Err(anyhow!("{e}")));
                    }
                },
                _ => return None,
            }
        }
    }
}

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
