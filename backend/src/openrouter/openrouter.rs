use std::sync::{Arc, RwLock};

use crate::openrouter::StreamCompletion;

use super::{Model, error::Error, raw};

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
    pub(super) api_key: String,
    pub(super) chat_completion_endpoint: String,
    model_ids: Arc<RwLock<Vec<String>>>,
    pub(super) http_client: reqwest::Client,
    // true if not openrouter
    pub(super) compatibility_mode: bool,
}

impl Openrouter {
    pub(super) fn get_request(&self, insert_web_search_context: bool) -> raw::CompletionReq {
        let mut default_req = raw::CompletionReq::default();
        if self.compatibility_mode {
            default_req.usage = None;
        } else {
            default_req.plugins.push(raw::Plugin::pdf());
            if insert_web_search_context {
                default_req.plugins.push(raw::Plugin::web());
            }
        }
        default_req
    }
    pub fn new(api_key: impl AsRef<str>, api_base: impl AsRef<str>) -> Self {
        let api_base = api_base.as_ref();
        let api_key = api_key.as_ref().to_string();

        let chat_completion_endpoint =
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'));

        log::info!(
            "Using endpoint {} for completions",
            &chat_completion_endpoint
        );

        let compatibility_mode = !api_base.contains("openrouter");
        if compatibility_mode {
            log::warn!("Custom API_BASE detected, disabling plugin support");
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
            model_ids,
            http_client: reqwest::Client::new(),
            compatibility_mode,
        }
    }

    /// Get a list of available model IDs
    pub fn get_model_ids(&self) -> Vec<String> {
        self.model_ids.read().unwrap().clone()
    }

    pub async fn stream(
        &self,
        model: Model,
        messages: Vec<Message>,
        tools: Vec<Tool>,
    ) -> Result<StreamCompletion, Error> {
        let tools: Vec<raw::Tool> = tools.into_iter().map(|t| t.into()).collect();
        let mut messages = messages;

        log::debug!("start streaming with model {}", &model.id);

        #[cfg(debug_assertions)]
        check_message(&messages);

        // https://openrouter.ai/docs/api-reference/overview#assistant-prefill
        if matches!(messages.last(), Some(Message::Assistant { .. })) {
            messages.push(Message::User("".to_string()));
        }

        let mut req = raw::CompletionReq {
            messages: messages
                .into_iter()
                .map(|m| m.to_raw_message(&model.id))
                .collect(),
            model: model.id.clone(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            tools,
            response_format: model.response_format.clone(),
            ..self.get_request(false) // Web search plugin seems to break annotation/preserved reasoning blocks, it's disable for now
        };

        req.log();

        StreamCompletion::request(
            &self.http_client,
            &self.api_key,
            &self.chat_completion_endpoint,
            req,
        )
        .await
    }

    /// Use [`super::model::ModelBuilder`] instead.
    ///
    /// complete without streaming
    ///
    /// Note that we enforce low thinking budget and 512 output token,
    /// primarily because it's used by title generation
    #[deprecated]
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
        if matches!(messages.last(), Some(Message::Assistant { .. })) {
            messages.push(Message::User("".to_string()));
        }

        let mut reasoning = None;
        let mut max_tokens = None;
        if !self.compatibility_mode {
            max_tokens = Some(512);
            reasoning = Some(raw::Reasoning::low());
        }

        let req = raw::CompletionReq {
            messages: messages
                .into_iter()
                .map(|m| m.to_raw_message(&model.id))
                .collect(),
            model: model.id.clone(),
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            max_tokens,
            stream: false,
            response_format: model.response_format.clone(),
            reasoning,
            ..self.get_request(false)
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
            .map(|u| {
                (
                    u.total_tokens,
                    u.cost_details
                        .map(|x| x.upstream_inference_cost)
                        .flatten()
                        .unwrap_or(u.cost),
                )
            })
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
    Assistant {
        content: String,
        annotations: Option<serde_json::Value>,
        reasoning_details: Option<serde_json::Value>,
    },
    MultipartUser {
        text: String,
        files: Vec<File>,
    },
    ToolCall(MessageToolCall),
    ToolResult(MessageToolResult),
}

impl Message {
    pub fn to_raw_message(self, target_model_id: &str) -> raw::Message {
        match self {
            Message::Assistant {
                content,
                annotations,
                reasoning_details,
            } => {
                let mut reasoning_details_value = None;
                if let Some(details) = reasoning_details {
                    if let Some(obj) = details.as_object() {
                        if let Some(model_id) = obj.get("model_id").and_then(|v| v.as_str()) {
                            if target_model_id.starts_with(model_id) {
                                reasoning_details_value = obj.get("data").cloned();
                            }
                        }
                    }
                }

                raw::Message {
                    role: raw::Role::Assistant,
                    content: Some(content),
                    annotations,
                    reasoning_details: reasoning_details_value.map(|v| vec![v]).unwrap_or_default(),
                    ..Default::default()
                }
            }
            _ => self.into(),
        }
    }
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
            Message::Assistant {
                content,
                annotations,
                reasoning_details: _,
            } => raw::Message {
                role: raw::Role::Assistant,
                content: Some(content),
                annotations,
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
pub(super) fn check_message(message: &[Message]) {
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
