use super::message::*;
use crate::openrouter::{StreamCompletion, option::CompletionOption};

use super::capability::CapabilityResolver;
use super::model_cache::ModelCacheManager;
use super::{Model, error::Error, raw};
use protocol::OcrEngine;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub struct Openrouter {
    pub(super) api_key: String,
    pub(super) chat_completion_endpoint: String,
    pub(super) embedding_endpoint: String,
    model_cache: ModelCacheManager,
    http_client: reqwest::Client,
    compatibility_mode: bool,
}

impl Openrouter {
    async fn create_request(
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

        let capability = self.get_capability(&model).await;

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

            if capability.image_output {
                modalities.push("image".to_string());
            }

            if capability.text_output {
                modalities.push("text".to_string());
            }
        }

        let temperature = match option.temperature {
            Some(t) => Some(t),
            None => model.temperature,
        };

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
            temperature,
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

        let embedding_endpoint = format!("{}/v1/embeddings", api_base.trim_end_matches('/'));
        let chat_completion_endpoint =
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'));
        let models_endpoint = format!("{}/v1/models", api_base.trim_end_matches('/'));

        log::info!(
            "Using endpoint {} for completions",
            &chat_completion_endpoint
        );

        let compatibility_mode = !api_base.contains("openrouter");
        if compatibility_mode {
            log::warn!("Custom API_BASE detected, disabling plugin support");
        }

        let model_cache = ModelCacheManager::new(models_endpoint, api_key.clone());

        Self {
            api_key,
            chat_completion_endpoint,
            embedding_endpoint,
            model_cache,
            http_client: reqwest::Client::new(),
            compatibility_mode,
        }
    }

    /// Get a list of available model IDs
    pub async fn get_model_ids(&self) -> Vec<String> {
        self.model_cache.get_model_ids().await
    }

    /// Get capability of a model (considers user overrides)
    pub async fn get_capability(&self, model: &Model) -> super::Capability {
        let resolver = CapabilityResolver::new(&self.model_cache);
        resolver.get_capability(model).await
    }

    pub async fn stream(
        &self,
        model: Model,
        messages: Vec<Message>,
        option: CompletionOption,
    ) -> Result<StreamCompletion, Error> {
        #[cfg(debug_assertions)]
        check_message(&messages);

        if !self.compatibility_mode {
            self.model_cache.ensure_model(&model.id).await?;
        }

        let req = self.create_request(messages, true, model, option).await;

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
            self.model_cache.ensure_model(&model.id).await?;
        }

        if !self.compatibility_mode {
            let capability = self.get_capability(&model).await;
            if !capability.text_output {
                return Err(Error::TextOutputNotSupported);
            }
        }

        option.image_generation = false;

        let req = self.create_request(messages, false, model, option).await;
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

        if !self.compatibility_mode {
            self.model_cache.ensure_model(&model.id).await?;
        }

        let structured_output = self.get_capability(&model).await.structured_output;

        let mut req = self.create_request(messages, false, model, option).await;

        if structured_output {
            let schema = schemars::schema_for!(T);
            let schema_json = serde_json::to_value(&schema).map_err(|e| Error::Serde(e))?;

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

    pub async fn embed(&self, model: &str, input: &[String]) -> Result<Embedding, Error> {
        if input.is_empty() {
            return Ok(Embedding {
                price: 0.0,
                response: Vec::new(),
            });
        }
        let req = raw::EmbeddingBatchReq {
            model: model.to_string(),
            input: input.to_vec(),
        };
        let res = self
            .http_client
            .post(&self.embedding_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .json(&req)
            .send()
            .await
            .map_err(Error::Http)?;

        let mut result: raw::EmbeddingResponse = res.json().await.map_err(Error::Http)?;

        result.data.sort_by(|a, b| a.index.cmp(&b.index));

        let response = result
            .data
            .into_iter()
            .map(|embedding| embedding.embedding)
            .collect();

        Ok(Embedding {
            price: 0.0,
            response,
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

pub struct Embedding {
    pub price: f64,
    pub response: Vec<Vec<f32>>,
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
