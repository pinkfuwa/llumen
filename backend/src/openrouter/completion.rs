use crate::openrouter::error::Error;
use crate::openrouter::raw;
use crate::openrouter::stream::StreamCompletion;

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
    pub response_format: Option<raw::ResponseFormat>,
}

impl Model {
    pub fn get_model_id(&self) -> String {
        let mut id = self.id.clone();
        if self.online {
            id.push_str(":online");
        }
        id
    }

    pub fn builder(id: impl Into<String>) -> ModelBuilder {
        ModelBuilder::new(id)
    }
}

pub struct ModelBuilder {
    id: String,
    temperature: Option<f32>,
    repeat_penalty: Option<f32>,
    top_k: Option<i32>,
    top_p: Option<f32>,
    online: bool,
    response_format: Option<raw::ResponseFormat>,
}

impl ModelBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            online: false,
            response_format: None,
        }
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = Some(repeat_penalty);
        self
    }

    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    pub fn response_format(mut self, response_format: raw::ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn json_schema(mut self, name: impl Into<String>, schema: serde_json::Value) -> Self {
        self.response_format = Some(raw::ResponseFormat {
            r#type: "json_schema".to_string(),
            json_schema: serde_json::json!({
                "name": name.into(),
                "strict": true,
                "schema": schema
            }),
        });
        self
    }

    pub fn build(self) -> Model {
        Model {
            id: self.id,
            temperature: self.temperature,
            repeat_penalty: self.repeat_penalty,
            top_k: self.top_k,
            top_p: self.top_p,
            online: self.online,
            response_format: self.response_format,
        }
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
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'));
        let mut default_req = raw::CompletionReq::default();

        log::info!(
            "Using endpoint {} for completions",
            &chat_completion_endpoint
        );

        if !api_base.contains("openrouter") {
            log::warn!("Custom API_BASE detected, disabling plugin support");
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
    ) -> impl std::future::Future<Output = Result<StreamCompletion, Error>> {
        log::debug!("start streaming with model {}", &model.id);

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
            response_format: model.response_format.clone(),
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
    ) -> Result<ChatCompletion, Error> {
        log::debug!("start completion with model {}", &model.id);

        if model.online {
            log::warn!("Online models should not be used in non-streaming completions.");
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
            response_format: model.response_format.clone(),
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
            .map_err(Error::Http)?;

        let json = res
            .json::<raw::CompletionResponse>()
            .await
            .map_err(Error::Http)?;

        if let Some(error) = json.error {
            log::warn!("openrouter finish with api error: {}", &error.message);
            return Err(Error::from(error));
        }

        let choice =
            json.choices
                .unwrap_or_default()
                .into_iter()
                .next()
                .ok_or(Error::MalformedResponse(
                    "No choices in completion response",
                ))?;

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
    AssistantAnnotationed {
        text: String,
        annotations: serde_json::Value,
    },
    MultipartUser {
        text: String,
        files: Vec<File>,
    },
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
            Message::AssistantAnnotationed { text, annotations } => raw::Message {
                role: raw::Role::Assistant,
                content: Some(text),
                annotations: Some(annotations),
                ..Default::default()
            },
            Message::MultipartUser { text, files } => {
                let files = files
                    .into_iter()
                    .flat_map(|f| {
                        let (first, second) = raw::MessagePart::unknown(&f.name, f.data);
                        vec![first, second]
                    })
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

pub fn check_message(message: &[Message]) {
    #[cfg(not(debug_assertions))]
    return;

    // For each ToolResult, check for ToolCall and its order
    let mut result_ids = message
        .iter()
        .filter(|m| matches!(m, Message::ToolResult(_)))
        .map(|m| match m {
            Message::ToolResult(result) => result.id.as_str(),
            _ => unreachable!(),
        });

    let mut call_ids = message
        .iter()
        .filter(|m| matches!(m, Message::ToolCall(_)))
        .map(|m| match m {
            Message::ToolCall(call) => call.id.as_str(),
            _ => unreachable!(),
        });

    loop {
        match (call_ids.next(), result_ids.next()) {
            (Some(call), Some(result)) => {
                assert_eq!(result, call, "ToolResult and ToolCall IDs do not match")
            }
            (None, None) => break,
            _ => panic!("Mismatched ToolCall and ToolResult IDs"),
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
