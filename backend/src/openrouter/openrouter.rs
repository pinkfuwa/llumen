use std::sync::{Arc, RwLock};

use crate::openrouter::{StreamCompletion, option::CompletionOption};

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
    fn create_request(
        &self,
        mut messages: Vec<Message>,
        stream: bool,
        model: Model,
        option: CompletionOption,
    ) -> raw::CompletionReq {
        // https://openrouter.ai/docs/api-reference/overview#assistant-prefill
        if matches!(messages.last(), Some(Message::Assistant { .. })) {
            messages.push(Message::User("".to_string()));
        }

        let mut plugins = Vec::new();
        let mut modalities = Vec::new();

        if !self.compatibility_mode {
            plugins.push(raw::Plugin::pdf());
            if option.insert_web_search_context {
                log::debug!("inserting web search context");
                plugins.push(raw::Plugin::web());
            }
            if option.image_generation {
                modalities.extend(["text".to_string(), "image".to_string()]);
            }
        }

        let usage = if self.compatibility_mode {
            None
        } else {
            Some(raw::UsageReq { include: true })
        };

        let reasoning = option
            .reasoning_effort
            .to_value()
            .map(|effort| raw::Reasoning { effort });

        let tools: Vec<raw::Tool> = option.tools.into_iter().map(|t| t.into()).collect();

        raw::CompletionReq {
            model: model.id.clone(),
            messages: messages
                .into_iter()
                .map(|m| m.to_raw_message(&model.id))
                .collect(),
            stream,
            temperature: model.temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            max_tokens: option.max_tokens,
            tools,
            plugins,
            usage,
            reasoning,
            modalities,
            response_format: None,
        }
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
        option: CompletionOption,
    ) -> Result<StreamCompletion, Error> {
        #[cfg(debug_assertions)]
        check_message(&messages);

        let req = self.create_request(messages, true, model, option);

        req.log();

        StreamCompletion::request(
            &self.http_client,
            &self.api_key,
            &self.chat_completion_endpoint,
            req,
        )
        .await
    }

    async fn send_complete_request(
        &self,
        req: raw::CompletionReq,
    ) -> Result<ChatCompletion, Error> {
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

    pub async fn complete(
        &self,
        messages: Vec<Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<ChatCompletion, Error> {
        debug_assert!(
            !option.image_generation,
            "Image generation supported only on streaming"
        );

        let req = self.create_request(messages, false, model, option);
        req.log();

        self.send_complete_request(req).await
    }

    pub async fn structured<T>(
        &self,
        messages: Vec<Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<StructuredCompletion<T>, Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        debug_assert!(
            !option.image_generation,
            "Image generation supported only on streaming"
        );

        let structured_output = model.capabilities.structured_output;

        let mut req = self.create_request(messages, false, model, option);
        req.log();

        if structured_output {
            let schema = schemars::schema_for!(T);
            let schema_json = serde_json::to_value(&schema).map_err(|e| Error::Serde(e))?;
            req.response_format = Some(raw::ResponseFormat {
                r#type: "json_schema".to_string(),
                json_schema: serde_json::json!({
                    "name": "result",
                    "strict": true,
                    "schema": schema_json
                }),
            });
        }

        let completion = self.send_complete_request(req).await?;
        let result: T = serde_json::from_str(&completion.response).map_err(Error::Serde)?;
        Ok(StructuredCompletion {
            price: completion.price,
            token: completion.token,
            response: result,
        })
    }
}

pub struct ChatCompletion {
    pub price: f64,
    pub token: usize,
    pub response: String,
}

pub struct StructuredCompletion<T> {
    pub price: f64,
    pub token: usize,
    pub response: T,
}

#[derive(Debug, Clone)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>,
}

// generated image
#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub mime_type: String,
}

impl Image {
    /// Parse a data URL like "data:image/png;base64,iVBORw0KGgo..."
    pub fn from_data_url(url: &str) -> Result<Self, Error> {
        if !url.starts_with("data:") {
            return Err(Error::MalformedResponse(
                "Image URL does not start with 'data:'",
            ));
        }

        let url = url.strip_prefix("data:").unwrap();

        let parts: Vec<&str> = url.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(Error::MalformedResponse("Invalid data URL format"));
        }

        let metadata = parts[0];
        let base64_data = parts[1];

        // Parse metadata like "image/png;base64"
        let mime_type = if let Some(semicolon_pos) = metadata.find(';') {
            metadata[..semicolon_pos].to_string()
        } else {
            metadata.to_string()
        };

        // Decode base64
        let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, base64_data)
            .map_err(|_| Error::MalformedResponse("Failed to decode base64 image data"))?;

        Ok(Image { data, mime_type })
    }

    pub fn from_raw_image(raw_image: super::raw::Image) -> Result<Self, Error> {
        Self::from_data_url(&raw_image.image_url.url)
    }
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
        images: Vec<Image>,
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
                images,
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
                if images.is_empty() {
                    return raw::Message {
                        role: raw::Role::Assistant,
                        content: Some(content),
                        annotations,
                        reasoning_details: reasoning_details_value
                            .map(|v| vec![v])
                            .unwrap_or_default(),
                        ..Default::default()
                    };
                }
                let mut parts = Vec::new();

                for image in images {
                    let data_url = format!(
                        "data:{};base64,{}",
                        image.mime_type,
                        base64::Engine::encode(
                            &base64::engine::general_purpose::STANDARD,
                            &image.data
                        )
                    );
                    parts.push(raw::MessagePart::image_url(data_url));
                }

                parts.push(raw::MessagePart::text(content));

                raw::Message {
                    role: raw::Role::Assistant,
                    contents: Some(parts),
                    annotations,
                    reasoning_details: reasoning_details_value.map(|v| vec![v]).unwrap_or_default(),
                    ..Default::default()
                }
            }
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
                role: raw::Role::Assistant,
                tool_calls: Some(vec![raw::ToolCallReq {
                    id,
                    function: raw::ToolFunctionResp {
                        name: Some(name),
                        arguments: Some(arguments),
                    },
                    r#type: "function".to_string(),
                }]),
                content: Some("".to_string()),
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
#[allow(dead_code)]
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
