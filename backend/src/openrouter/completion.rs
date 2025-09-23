use anyhow::{Context, Result};

use super::raw;
use super::stream::StreamCompletion;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

#[derive(Clone, Default)]
pub struct Model {
    pub id: String,
    pub temperature: Option<f32>,
    pub repeat_penalty: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub online: bool,
}

impl Model {
    pub fn get_model_id(&self) -> String {
        let mut id = self.id.clone();
        if self.online {
            id.push_str(":online");
        }
        return id;
    }
}

pub struct Openrouter {
    api_key: String,
    chat_completion_endpoint: String,
    default_req: raw::CompletionReq,
    http_client: reqwest::Client,
}

impl Openrouter {
    pub fn new(api_key: impl AsRef<str>, api_base: impl AsRef<str>) -> Self {
        let api_base = api_base.as_ref();
        let api_key = api_key.as_ref().to_string();
        let chat_completion_endpoint =
            format!("{}/api/v1/chat/completions", api_base.trim_end_matches('/'));
        let mut default_req = raw::CompletionReq::default();

        if !api_base.contains("openrouter") {
            tracing::warn!("Custom API_BASE detected, disabling plugin support");
            default_req.plugins = None;
            default_req.usage = None;
        }

        Self {
            api_key,
            chat_completion_endpoint,
            default_req,
            http_client: reqwest::Client::new(),
        }
    }
    pub fn stream(
        &self,
        mut messages: Vec<Message>,
        model: &Model,
        tools: Vec<Tool>,
    ) -> impl std::future::Future<Output = Result<StreamCompletion>> {
        tracing::debug!("start streaming with model {}", &model.id);

        let tools = match tools.is_empty() {
            true => None,
            false => Some(tools.into_iter().map(|t| t.into()).collect()),
        };

        // https://openrouter.ai/docs/api-reference/overview#assistant-prefill
        if matches!(messages.last(), Some(Message::Assistant(_))) {
            messages.push(Message::User("".to_string()));
        }

        let req = raw::CompletionReq {
            messages: messages.into_iter().map(|m| m.into()).collect(),
            model: model.get_model_id(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            tools,
            ..self.default_req.clone()
        };

        req.log();

        StreamCompletion::request(
            &self.http_client,
            &self.api_key,
            &self.chat_completion_endpoint,
            req,
        )
    }
    pub async fn complete(
        &self,
        mut messages: Vec<Message>,
        model: Model,
    ) -> Result<ChatCompletion> {
        tracing::debug!("start completion with model {}", &model.id);

        if model.online {
            tracing::warn!("Online models should not be used in non-streaming completions.");
        }

        // https://openrouter.ai/docs/api-reference/overview#assistant-prefill
        if matches!(messages.last(), Some(Message::Assistant(_))) {
            messages.push(Message::User("".to_string()));
        }

        let req = raw::CompletionReq {
            messages: messages.into_iter().map(|m| m.into()).collect(),
            model: model.get_model_id(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            stream: false,
            ..self.default_req.clone()
        };

        req.log();

        let res = self
            .http_client
            .post(&self.chat_completion_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req)
            .send()
            .await
            .map_err(|err| {
                tracing::warn!("openrouter finish with error: {}", &err);
                err
            })
            .context("Failed to build request")?;

        let json = res
            .json::<raw::CompletionResponse>()
            .await
            .context("Failed to parse response")?;

        if let Some(error) = json.error {
            tracing::warn!("openrouter finish with api error: {}", &error.message);
            return Err(anyhow::anyhow!("Openrouter API error: {}", error.message));
        }

        let choice = json
            .choices
            .unwrap_or(Vec::new())
            .into_iter()
            .next()
            .context("Malformed response")?;

        let text = choice.message.content;

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

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct MessageToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub struct MessageToolResult {
    pub id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    System(String),
    User(String),
    Assistant(String),
    MultipartUser { text: String, files: Vec<File> },
    ToolCall(MessageToolCall),
    ToolResult(MessageToolResult),
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
            Message::ToolCall(MessageToolCall {
                id,
                name,
                arguments,
            }) => raw::Message {
                role: raw::Role::Tool,
                tool_calls: Some(vec![raw::ToolCallReq {
                    id,
                    function: raw::ToolFunctionResp {
                        name: Some(name),
                        arguments: Some(arguments),
                    },
                    r#type: "function".to_string(),
                }]),
                ..Default::default()
            },
            Message::ToolResult(MessageToolResult { id, content }) => raw::Message {
                role: raw::Role::Tool,
                content: Some(content),
                tool_call_id: Some(id),
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone)]
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
