use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::message::*;
use crate::openrouter::{StreamCompletion, option::CompletionOption};

use super::{Model, error::Error, raw};
use protocol::OcrEngine;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

async fn fetch_models(url: &str, api_key: &str) -> Result<Vec<raw::Model>, Error> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .bearer_auth(api_key)
        .header("HTTP-Referer", HTTP_REFERER)
        .header("X-Title", X_TITLE)
        .send()
        .await?;

    let model: raw::ModelListResponse = response.json().await?;
    Ok(model.data)
}

pub struct Openrouter {
    pub(super) api_key: String,
    pub(super) chat_completion_endpoint: String,
    models: Arc<RwLock<HashMap<String, raw::Model>>>,
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

        let capability = self.get_capability(&model);

        let mut plugins = Vec::new();
        let mut modalities = Vec::new();

        if !self.compatibility_mode {
            match capability.ocr {
                OcrEngine::Native => plugins.push(raw::Plugin::pdf_native()),
                OcrEngine::Text => plugins.push(raw::Plugin::pdf_text()),
                OcrEngine::Mistral => plugins.push(raw::Plugin::mistral_ocr()),
                OcrEngine::Disabled => {}
            };
            if option.insert_web_search_context {
                log::debug!("inserting web search context");
                plugins.push(raw::Plugin::web());
            }
            if option.image_generation && capability.image_output {
                modalities.extend(["text".to_string(), "image".to_string()]);
            }
        }

        let usage = if self.compatibility_mode {
            None
        } else {
            Some(raw::UsageReq { include: true })
        };

        let reasoning = match (self.compatibility_mode, capability.reasoning) {
            (true, true) => raw::Reasoning {
                enabled: None,
                effort: option.reasoning_effort.to_value(),
            },
            (true, false) => raw::Reasoning::default(),
            (false, true) => raw::Reasoning {
                enabled: Some(true),
                effort: option.reasoning_effort.to_value(),
            },
            (false, false) => raw::Reasoning {
                effort: None,
                enabled: Some(false),
            },
        };

        let tools: Vec<raw::Tool> = option.tools.into_iter().map(|t| t.into()).collect();

        raw::CompletionReq {
            model: model.id.clone(),
            messages: messages
                .into_iter()
                .map(|m| m.to_raw_message(&model.id, &capability))
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

        let models = Arc::new(RwLock::new(HashMap::new()));

        {
            let models = models.clone();
            let api_key = api_key.clone();
            let endpoint = format!("{}/v1/models", api_base.trim_end_matches('/'));
            tokio::spawn(async move {
                match fetch_models(&endpoint, &api_key).await {
                    Ok(model_list) => {
                        log::info!("{} models available", model_list.len());
                        let map: HashMap<String, raw::Model> =
                            model_list.into_iter().map(|m| (m.id.clone(), m)).collect();
                        *models.write().unwrap() = map;
                    }
                    Err(err) => log::error!("Failed to fetch models: {}", err),
                }
            });
        }

        Self {
            api_key,
            chat_completion_endpoint,
            models,
            http_client: reqwest::Client::new(),
            compatibility_mode,
        }
    }

    /// Get a list of available model IDs
    pub fn get_model_ids(&self) -> Vec<String> {
        self.models.read().unwrap().keys().cloned().collect()
    }

    /// get capability of a model(consider user overrides)
    pub fn get_capability(&self, model: &Model) -> super::Capability {
        let overrides: super::MaybeCapability = model.capability.clone().into();
        let capability = self.get_openrouter_capability(&model.id);

        macro_rules! merge {
            ($v:ident) => {
                match overrides.$v {
                    Some(v) => v,
                    None => capability.$v.unwrap_or(true),
                }
            };
        }
        super::Capability {
            image_output: merge!(image_output),
            image_input: merge!(image_input),
            structured_output: merge!(structured_output),
            toolcall: merge!(toolcall),
            ocr: match overrides.ocr {
                Some(v) => v,
                None => capability.ocr.unwrap_or(OcrEngine::Disabled),
            },
            audio: merge!(audio),
            reasoning: merge!(reasoning),
        }
    }

    /// get openrouter capabilities
    fn get_openrouter_capability(&self, model_id: &str) -> super::MaybeCapability {
        if self.compatibility_mode {
            return super::MaybeCapability::default();
        }

        let models = self.models.read().unwrap();
        let model = models.get(model_id);

        if model.is_none() {
            return super::MaybeCapability::default();
        }

        let model = model.unwrap();

        let supports_file_modality = model
            .architecture
            .input_modalities
            .contains(&raw::Modality::File);

        super::MaybeCapability {
            image_output: Some(
                model
                    .architecture
                    .output_modalities
                    .contains(&raw::Modality::Image),
            ),
            image_input: Some(
                model
                    .architecture
                    .input_modalities
                    .contains(&raw::Modality::Image),
            ),
            structured_output: Some(
                model
                    .supported_parameters
                    .contains(&raw::SupportedParams::StructuredOutput),
            ),
            toolcall: Some(
                model
                    .supported_parameters
                    .contains(&raw::SupportedParams::Tools),
            ),
            ocr: Some(if supports_file_modality {
                OcrEngine::Native
            } else {
                OcrEngine::Text
            }),
            audio: Some(
                model
                    .architecture
                    .input_modalities
                    .contains(&raw::Modality::Audio),
            ),
            reasoning: Some(
                model
                    .supported_parameters
                    .contains(&raw::SupportedParams::Reasoning),
            ),
        }
    }

    /// Check if a model supports tools (function calling)
    /// Returns None if model not found or not using OpenRouter
    pub fn supports_tools(&self, model_id: &str) -> Option<bool> {
        if self.compatibility_mode {
            return None;
        }

        let models = self.models.read().unwrap();
        let model = models.get(model_id)?;

        Some(
            model
                .supported_parameters
                .contains(&raw::SupportedParams::Tools),
        )
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
        mut option: CompletionOption,
    ) -> Result<ChatCompletion, Error> {
        debug_assert!(
            !option.image_generation,
            "Image generation supported only on streaming"
        );

        if !self.compatibility_mode {
            const IMAGE_ONLY_PREFIXES: &[&str] = &["black-forest-labs/", "sourceful/"];
            if IMAGE_ONLY_PREFIXES
                .iter()
                .any(|prefix| model.id.starts_with(prefix))
            {
                return Ok(ChatCompletion::new());
            }

            let models = self.models.read().unwrap();
            if let Some(model) = models.get(&model.id) {
                if !model
                    .architecture
                    .output_modalities
                    .contains(&raw::Modality::Text)
                {
                    return Ok(ChatCompletion::new());
                }
            }
        }

        option.image_generation = false;

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

        // to determine does it support structured output,
        // first look for capabilities override(capabilities from model)
        // Then check if openrouter is used, trust what openrouter give us
        // If not openrouter and no override, assume supported.
        let structured_output = self.get_capability(&model).structured_output;

        let mut req = self.create_request(messages, false, model, option);

        if structured_output {
            let schema = schemars::schema_for!(T);
            let schema_json = serde_json::to_value(&schema).map_err(|e| Error::Serde(e))?;

            // structure need to be marked with `#[schemars(deny_unknown_fields)]`

            req.response_format = Some(raw::ResponseFormat {
                r#type: "json_schema".to_string(),
                json_schema: serde_json::json!({
                    "name": std::any::type_name::<T>().split("::").last().unwrap(),
                    "strict": true,
                    "schema": schema_json
                }),
            });
        }

        req.log();

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

impl ChatCompletion {
    pub fn new() -> Self {
        ChatCompletion {
            price: 0.0,
            token: 0,
            response: String::new(),
        }
    }
}

pub struct StructuredCompletion<T> {
    pub price: f64,
    pub token: usize,
    pub response: T,
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
