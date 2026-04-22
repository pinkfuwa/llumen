use protocol::ReasoningEffort;
use stream_json::IntoSerializer;

use super::listing::ModelListing;
use super::{error::Error, raw, CompletionOption, Capability, MaybeCapability, Model};

#[derive(Clone)]
pub(super) struct ChatClient {
    api_key: String,
    chat_completion_endpoint: String,
    http_client: reqwest::Client,
    is_custom_api: bool,
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

impl ChatClient {
    pub fn new(
        api_key: String,
        chat_completion_endpoint: String,
        http_client: reqwest::Client,
        is_custom_api: bool,
    ) -> Self {
        Self {
            api_key,
            chat_completion_endpoint,
            http_client,
            is_custom_api,
        }
    }

    fn split_model_id(model_id: &str) -> &str {
        model_id.split(':').next().unwrap_or(model_id)
    }

    async fn get_openrouter_capability(
        &self,
        listing: &ModelListing,
        model_id: &str,
    ) -> Capability {
        listing
            .get(model_id)
            .await
            .map(|capability| Capability {
                reasoning_effort: ReasoningEffort::Auto,
                ..capability.into()
            })
            .unwrap_or_else(|| Capability {
                text_output: true,
                reasoning_effort: ReasoningEffort::Auto,
                video_input: false,
                ..Default::default()
            })
    }

    fn merge_capability(base: Capability, overrides: MaybeCapability) -> Capability {
        macro_rules! merge {
            ($field:ident) => {
                match overrides.$field {
                    Some(value) => value,
                    None => base.$field,
                }
            };
        }

        Capability {
            text_output: merge!(text_output),
            image_output: merge!(image_output),
            image_input: merge!(image_input),
            video_input: merge!(video_input),
            structured_output: merge!(structured_output),
            toolcall: merge!(toolcall),
            ocr: merge!(ocr),
            audio: merge!(audio),
            reasoning: merge!(reasoning),
            reasoning_effort: merge!(reasoning_effort),
        }
    }

    async fn resolve_capability(&self, listing: &ModelListing, model: &Model) -> Capability {
        let model_id = Self::split_model_id(&model.id);
        let openrouter_capability = self.get_openrouter_capability(listing, model_id).await;
        Self::merge_capability(openrouter_capability, model.capability.clone())
    }

    async fn prepare_request(
        &self,
        listing: &ModelListing,
        mut messages: Vec<super::message::Message>,
        stream: bool,
        model: Model,
        option: CompletionOption,
    ) -> (raw::CompletionReq, Capability) {
        if matches!(
            messages.last(),
            Some(super::message::Message::Assistant { .. })
        ) {
            messages.push(super::message::Message::User(String::new()));
        }

        let capability = self.resolve_capability(listing, &model).await;

        let mut plugins = Vec::new();
        let mut modalities = Vec::new();

        let web_search_options = if option.insert_web_search_context && !self.is_custom_api {
            Some(raw::WebSearchOptions {
                search_context_size: "medium".to_string(),
            })
        } else {
            None
        };

        if !self.is_custom_api {
            match capability.ocr {
                protocol::OcrEngine::Native => plugins.push(raw::Plugin::pdf_native()),
                protocol::OcrEngine::Text => plugins.push(raw::Plugin::pdf_text()),
                protocol::OcrEngine::Mistral => plugins.push(raw::Plugin::mistral_ocr()),
                protocol::OcrEngine::Cloudflare => plugins.push(raw::Plugin::cloudflare_ocr()),
                protocol::OcrEngine::Disabled => {}
            }

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

        let temperature = option.temperature.or(model.temperature);

        let usage = if self.is_custom_api {
            None
        } else {
            Some(raw::UsageReq { include: true })
        };

        let reasoning_effort = if option.reasoning_max_tokens.is_some() {
            None
        } else if capability.reasoning {
            option.reasoning_effort.to_value()
        } else {
            None
        };

        let mut reasoning = raw::Reasoning {
            effort: reasoning_effort,
            enabled: Some(capability.reasoning),
            max_tokens: option.reasoning_max_tokens,
        };

        if self.is_custom_api {
            reasoning.set_compatible();
        }

        let mut tools = Vec::new();

        if capability.toolcall {
            tools.extend(option.tools.into_iter().map(Into::into));
        }

        let request = raw::CompletionReq {
            model: model.id.clone(),
            messages: messages
                .into_iter()
                .map(|message| message.to_raw_message(&model.id, &capability))
                .collect(),
            stream,
            temperature,
            repeat_penalty: model.repeat_penalty,
            top_k: model.top_k,
            top_p: model.top_p,
            max_tokens: option.max_tokens,
            tools,
            plugins,
            web_search_options,
            usage,
            reasoning,
            modalities,
            response_format: None,
            image_config: None,
            session_id: option.session_id,
        };

        (request, capability)
    }

    fn completion_body(req: raw::CompletionReq) -> (Option<usize>, reqwest::Body) {
        let size = req.size();
        let body = reqwest::Body::wrap_stream(req.into_stream());
        (size, body)
    }

    async fn send_complete_request(
        &self,
        req: raw::CompletionReq,
    ) -> Result<ChatCompletion, Error> {
        let session_id = req.session_id.clone();
        let (content_length, body) = Self::completion_body(req);
        let mut req_builder = self
            .http_client
            .post(&self.chat_completion_endpoint)
            .bearer_auth(&self.api_key)
            .header(super::HTTP_REFERER, super::LLUMEN_URL)
            .header(super::X_TITLE, super::LLUMEN_NAME)
            .header(http::header::CONTENT_TYPE, "application/json");

        if !self.is_custom_api {
            if let Some(ref sid) = session_id {
                req_builder = req_builder.header("x-session-id", sid);
            }
        }

        if let Some(len) = content_length {
            req_builder = req_builder.header(http::header::CONTENT_LENGTH, len);
        }

        let res = req_builder.body(body).send().await.map_err(Error::Http)?;

        let json = res
            .json::<raw::CompletionResponse>()
            .await
            .map_err(Error::Http)?;

        if let Some(error) = json.error {
            return Err(Error::from(error));
        }

        let (token, price) = json
            .usage
            .map(|usage| {
                (
                    usage.total_tokens,
                    usage
                        .cost_details
                        .and_then(|details| details.upstream_inference_cost)
                        .unwrap_or(usage.cost),
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

        Ok(ChatCompletion {
            price,
            token: token.unwrap_or_default() as usize,
            response: choice.message.content.unwrap_or_default(),
        })
    }

    pub(super) async fn get_capability(&self, listing: &ModelListing, model: &Model) -> Capability {
        self.resolve_capability(listing, model).await
    }

    pub(super) async fn stream(
        &self,
        listing: &ModelListing,
        messages: Vec<super::message::Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<super::stream::StreamCompletion, Error> {
        let (request, _) = self
            .prepare_request(listing, messages, true, model, option)
            .await;

        super::stream::StreamCompletion::request(
            &self.http_client,
            &self.api_key,
            &self.chat_completion_endpoint,
            self.is_custom_api,
            request,
        )
        .await
    }

    pub(super) async fn complete(
        &self,
        listing: &ModelListing,
        messages: Vec<super::message::Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<ChatCompletion, Error> {
        debug_assert!(
            !option.image_generation,
            "Image generation supported only on streaming"
        );

        let (request, capability) = self
            .prepare_request(listing, messages, false, model, option)
            .await;

        if !self.is_custom_api && !capability.text_output {
            return Err(Error::TextOutputNotSupported);
        }

        self.send_complete_request(request).await
    }

    pub(super) async fn structured<T>(
        &self,
        listing: &ModelListing,
        messages: Vec<super::message::Message>,
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

        let (mut request, capability) = self
            .prepare_request(listing, messages, false, model, option)
            .await;

        if capability.structured_output {
            let schema = schemars::schema_for!(T);
            let schema_json = serde_json::to_value(&schema).map_err(Error::Serde)?;

            request.response_format = Some(raw::ResponseFormat {
                r#type: "json_schema".to_string(),
                json_schema: serde_json::json!({
                    "name": std::any::type_name::<T>().split("::").last().unwrap_or("response"),
                    "strict": true,
                    "schema": schema_json
                }),
            });
        }

        let completion = self.send_complete_request(request).await?;
        let response: T = serde_json::from_str(&completion.response).map_err(Error::Serde)?;

        Ok(StructuredCompletion {
            price: completion.price,
            token: completion.token,
            response,
        })
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub(super) fn check_message(message: &[super::message::Message]) {
    let mut result_ids = message
        .iter()
        .filter(|m| matches!(m, super::message::Message::ToolResult(_)))
        .map(|m| match m {
            super::message::Message::ToolResult(result) => result.id.as_str(),
            _ => unreachable!(),
        });

    let mut call_ids = message
        .iter()
        .filter(|m| matches!(m, super::message::Message::ToolCall(_)))
        .map(|m| match m {
            super::message::Message::ToolCall(call) => call.id.as_str(),
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
