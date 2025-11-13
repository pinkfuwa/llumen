use std::sync::{Arc, RwLock};

use super::{Model, StreamCompletion, error::Error, raw};

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

async fn fetch_models(url: &str, api_key: &str) -> Result<Vec<String>, Error> {
    #[derive(serde::Deserialize)]
    struct Model {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct Response {
        data: Vec<Model>,
    }

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .bearer_auth(api_key)
        .header("HTTP-Referer", HTTP_REFERER)
        .header("X-Title", X_TITLE)
        .send()
        .await?;

    let model: Response = response.json().await?;
    Ok(model.data.into_iter().map(|m| m.id).collect())
}

pub struct Openrouter {
    api_key: String,
    chat_completion_endpoint: String,
    model_ids: Arc<RwLock<Vec<String>>>,
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

        let model_ids = Arc::new(RwLock::new(Vec::new()));

        {
            let model_ids = model_ids.clone();
            let api_key = api_key.clone();
            let endpoint = format!("{}/v1/models", api_base.trim_end_matches('/'));
            tokio::spawn(async move {
                match fetch_models(&endpoint, &api_key).await {
                    Ok(models) => {
                        log::info!("{} models available", models.len());
                        *model_ids.write().unwrap() = models;
                    }
                    Err(err) => log::error!("Failed to fetch models: {}", err),
                }
            });
        }

        Self {
            api_key,
            chat_completion_endpoint,
            default_req,
            model_ids,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get a list of available model IDs
    pub fn get_model_ids(&self) -> Vec<String> {
        self.model_ids.read().unwrap().clone()
    }

    /// Use [`super::builder::ModelBuilder`] instead.
    #[deprecated]
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

        #[cfg(debug_assertions)]
        check_message(&messages);

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

        debug_assert!(
            !model.online,
            "Online models should not be used in non-streaming completions."
        );
        debug_assert!(
            !messages.iter().any(|x| matches!(x, Message::ToolCall(_))),
            "Tool calls should not be used in non-streaming completions."
        );

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

        let (token, price) = json
            .usage
            .map(|x| (x.total_tokens, x.cost))
            .unwrap_or_default();

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
            price,
            token: token.unwrap_or_default() as usize,
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

#[cfg(debug_assertions)]
pub fn check_message(message: &[Message]) {
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
