use anyhow::{Context, Result};
use dotenv::var;
use reqwest::Client;

use super::raw;
use super::stream::StreamCompletion;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub struct Openrouter {
    api_key: String,
    chat_completion_endpoint: String,
    default_req: raw::CompletionReq,
}

impl Openrouter {
    pub fn new() -> Self {
        let api_key = var("API_KEY").expect("API_KEY is required");
        let api_base = var("API_BASE").unwrap_or("https://openrouter.ai/".to_string());
        let chat_completion_endpoint =
            format!("{}/api/v1/chat/completions", api_base.trim_end_matches('/'));
        let mut default_req = raw::CompletionReq::default();

        if !api_base.contains("openrouter") {
            default_req.plugins = None;
        }

        Self {
            api_key,
            chat_completion_endpoint,
            default_req,
        }
    }
    pub fn stream(
        &self,
        messages: Vec<Message>,
        model: String,
        tools: Vec<Tool>,
    ) -> impl std::future::Future<Output = Result<StreamCompletion>> {
        let tools = match tools.is_empty() {
            true => None,
            false => Some(tools.into_iter().map(|t| t.into()).collect()),
        };

        let req = raw::CompletionReq {
            messages: messages.into_iter().map(|m| m.into()).collect(),
            model,
            tools,
            ..self.default_req.clone()
        };

        StreamCompletion::request(&self.api_key, &self.chat_completion_endpoint, req)
    }
    pub async fn complete(&self, messages: Vec<Message>, model: String) -> Result<ChatCompletion> {
        let req = raw::CompletionReq {
            messages: messages.into_iter().map(|m| m.into()).collect(),
            model,
            ..self.default_req.clone()
        };

        let res = Client::new()
            .post(&self.chat_completion_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req)
            .send()
            .await
            .context("Failed to build request")?;

        let json = res
            .json::<raw::CompletionResponse>()
            .await
            .context("Failed to parse response")?;

        let text = json
            .output
            .into_iter()
            .map(|msg| {
                msg.content
                    .into_iter()
                    .filter_map(|part| {
                        if let Some(kind) = part.r#type {
                            if kind.contains("text") {
                                return part.text;
                            }
                        }
                        None
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>()
            .join("");

        // TODO: calculate price and token usage
        Ok(ChatCompletion {
            price: 0.0,
            token: 0,
            response: text,
        })
    }
}

pub struct ChatCompletion {
    pub price: f64,
    pub token: usize,
    pub response: String,
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
