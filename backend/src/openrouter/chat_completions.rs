use anyhow::{Context, Result, anyhow};
use futures_util::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};

use super::raw;

pub struct Completion {
    source: EventSource,
}

impl Completion {
    pub async fn request(
        messages: Vec<Message>,
        model: String,
        api_key: &str,
        tools: Vec<Tool>,
    ) -> Result<Completion> {
        let tools = match tools.is_empty() {
            true => None,
            false => Some(tools.into_iter().map(|t| t.into()).collect()),
        };

        let req = raw::CompletionReq {
            messages: messages.into_iter().map(|m| m.into()).collect(),
            model,
            tools,
            ..Default::default()
        };

        let builder = Client::new()
            .post(" https://openrouter.ai/api/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&req);

        let source = EventSource::new(builder)?;

        Ok(Self { source })
    }

    pub fn close(&mut self) {
        self.source.close();
    }

    fn handle_choice(&mut self, choice: raw::Choice) -> CompletionResp {
        let delta = choice.delta;

        if let Some(reason) = choice.finish_reason {
            return match reason {
                raw::FinishReason::Stop => CompletionResp::ResponseToken(delta.content),
                raw::FinishReason::Length => CompletionResp::ResponseToken(delta.content),
                raw::FinishReason::ToolCalls => {
                    let tool_calls = delta.tool_calls.map(|x| x.into_iter().next()).flatten();
                    match tool_calls {
                        Some(tool_call) => CompletionResp::ToolCall {
                            name: tool_call.function.name,
                            args: tool_call.function.arguments,
                            id: tool_call.id,
                        },
                        None => CompletionResp::ResponseToken(delta.content),
                    }
                }
            };
        }
        if let Some(reasoning) = delta.reasoning {
            return CompletionResp::ReasoningToken(reasoning);
        }
        return CompletionResp::ResponseToken(delta.content);
    }

    fn handle_data(&mut self, data: &str) -> Result<CompletionResp> {
        // this approach made it compatible with both openrouter and openai
        if let Ok(resp) = serde_json::from_str::<raw::CompletionInfoResp>(data) {
            return Ok(CompletionResp::Usage {
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

    pub async fn next(&mut self) -> Option<Result<CompletionResp>> {
        loop {
            match self.source.next().await? {
                Ok(Event::Open) => continue,
                Ok(Event::Message(e)) if &e.data != "[DONE]" => {
                    return Some(self.handle_data(&e.data));
                }
                Err(e) => match e {
                    reqwest_eventsource::Error::StreamEnded => return None,
                    e => return Some(Err(anyhow!("{e}"))),
                },
                _ => return None,
            }
        }
    }
}

pub enum CompletionResp {
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

pub struct File {
    name: String,
    data: Vec<u8>,
}

pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    MultipartUser {
        text: String,
        files: Vec<File>,
    },
    ToolRequest {
        id: String,
        name: String,
        arguments: String,
    },
    ToolResult {
        id: String,
        content: String,
    },
}

impl From<Message> for raw::Message {
    fn from(msg: Message) -> Self {
        match msg {
            Message::System(msg) => raw::Message {
                role: raw::Role::System,
                content: Some(msg),
                ..Default::default()
            },
            Message::User(msg) => raw::Message {
                role: raw::Role::User,
                content: Some(msg),
                ..Default::default()
            },
            Message::Assistant(msg) => raw::Message {
                role: raw::Role::Assistant,
                content: Some(msg),
                ..Default::default()
            },
            Message::MultipartUser { text, files } => {
                let files = files
                    .into_iter()
                    .map(|f| {
                        let (first, second) = raw::MessagePart::unknown(&f.name, f.data);
                        vec![first, second]
                    })
                    .flatten()
                    .collect::<Vec<_>>();

                raw::Message {
                    role: raw::Role::User,
                    contents: Some(
                        std::iter::once(raw::MessagePart::text(text))
                            .chain(files.into_iter())
                            .collect(),
                    ),
                    ..Default::default()
                }
            }
            Message::ToolRequest {
                id,
                name,
                arguments,
            } => raw::Message {
                role: raw::Role::Tool,
                tool_calls: Some(vec![raw::ToolCallReq {
                    id,
                    function: raw::ToolFunctionResp { name, arguments },
                    r#type: "function".to_string(),
                }]),
                ..Default::default()
            },
            Message::ToolResult { id, content } => raw::Message {
                role: raw::Role::Tool,
                content: Some(content),
                tool_call_id: Some(id),
                ..Default::default()
            },
        }
    }
}

pub struct Tool {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

impl From<Tool> for raw::Tool {
    fn from(tool: Tool) -> Self {
        raw::Tool {
            r#type: "function".to_string(),
            function: raw::FunctionTool {
                name: tool.name,
                description: tool.description,
                parameters: tool.schema,
            },
        }
    }
}
