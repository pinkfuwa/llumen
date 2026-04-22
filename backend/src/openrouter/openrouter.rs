use std::sync::Arc;
use std::time::Duration;

use super::chat::ChatClient;
use super::image_gen::{AspectRatio, ImageGenClient};
use super::listing::{ModelListing, VideoModelListing};
use super::message::Message;
use super::raw;
use super::video_gen::{VideoGenClient, VideoGenerationOption, VideoModelCapability};
use super::{CompletionOption, Error, File, Model, StreamCompletion, LLUMEN_URL, LLUMEN_NAME};
use http::header::CONTENT_TYPE;
use stream_json::IntoSerializer;

pub struct Openrouter {
    pub(super) api_key: String,
    pub(super) embedding_endpoint: String,
    listing: Arc<ModelListing>,
    video_listing: Arc<VideoModelListing>,
    chat: ChatClient,
    image_gen: ImageGenClient,
    video_gen: VideoGenClient,
    http_client: reqwest::Client,
    is_custom_api: bool,
}

impl Openrouter {
    pub fn new(
        api_key: impl AsRef<str>,
        api_base: impl AsRef<str>,
        force_openrouter: bool,
    ) -> Self {
        let api_base = api_base.as_ref();
        let api_key = api_key.as_ref().to_string();

        let is_custom_api = if force_openrouter {
            false
        } else {
            !api_base.contains("openrouter")
        };
        if is_custom_api {
            log::warn!("Custom API_BASE detected, disabling plugin support");
        }

        let embedding_endpoint = format!("{}/v1/embeddings", api_base.trim_end_matches('/'));
        let chat_completion_endpoint =
            format!("{}/v1/chat/completions", api_base.trim_end_matches('/'));
        let videos_endpoint = format!("{}/v1/videos", api_base.trim_end_matches('/'));
        let video_models_endpoint = format!("{}/v1/videos/models", api_base.trim_end_matches('/'));
        let models_endpoint = match is_custom_api {
            true => format!("{}/v1/models", api_base.trim_end_matches('/')),
            false => {
                format!(
                    "{}/v1/models?output_modalities=text,image",
                    api_base.trim_end_matches('/')
                )
            }
        };

        log::info!(
            "Using endpoint {} for completions",
            &chat_completion_endpoint
        );

        let http_client = reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let listing = ModelListing::new(http_client.clone(), models_endpoint, api_key.clone());
        let video_listing =
            VideoModelListing::new(http_client.clone(), video_models_endpoint, api_key.clone());
        let chat = ChatClient::new(
            api_key.clone(),
            chat_completion_endpoint.clone(),
            http_client.clone(),
            is_custom_api,
        );
        let image_gen = ImageGenClient::new(
            api_key.clone(),
            chat_completion_endpoint.clone(),
            http_client.clone(),
        );
        let video_gen = VideoGenClient::new(
            api_key.clone(),
            videos_endpoint.clone(),
            http_client.clone(),
        );

        Self {
            api_key,
            embedding_endpoint,
            listing,
            video_listing,
            chat,
            image_gen,
            video_gen,
            http_client,
            is_custom_api,
        }
    }

    pub fn is_custom_api(&self) -> bool {
        self.is_custom_api
    }

    pub async fn get_model_ids(&self) -> Vec<String> {
        self.listing.get_model_ids().await
    }

    pub async fn get_video_model_ids(&self) -> Vec<String> {
        self.video_listing.get_model_ids().await
    }

    pub async fn get_video_model_capability(
        &self,
        model_id: &str,
    ) -> Result<VideoModelCapability, Error> {
        if !self.is_custom_api {
            self.video_listing.ensure(model_id).await?;
        }

        let model_id = model_id.split(':').next().unwrap_or(model_id);
        self.video_listing
            .get(model_id)
            .await
            .map(Into::into)
            .ok_or(Error::VideoGenModelNotFound)
    }

    pub async fn get_capability(&self, model: &Model) -> super::Capability {
        if !self.is_custom_api {
            let _ = self.listing.ensure(&model.id).await;
        }

        self.chat.get_capability(&self.listing, model).await
    }

    pub async fn stream(
        &self,
        model: Model,
        messages: Vec<Message>,
        option: CompletionOption,
    ) -> Result<StreamCompletion, Error> {
        #[cfg(debug_assertions)]
        super::chat::check_message(&messages);

        if !self.is_custom_api {
            self.listing.ensure(&model.id).await?;
        }

        self.chat
            .stream(&self.listing, messages, model, option)
            .await
    }

    pub async fn complete(
        &self,
        messages: Vec<Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<super::ChatCompletion, Error> {
        if !self.is_custom_api {
            self.listing.ensure(&model.id).await?;
        }

        self.chat
            .complete(&self.listing, messages, model, option)
            .await
    }

    pub async fn structured<T>(
        &self,
        messages: Vec<Message>,
        model: Model,
        option: CompletionOption,
    ) -> Result<super::StructuredCompletion<T>, Error>
    where
        T: serde::de::DeserializeOwned + schemars::JsonSchema,
    {
        if !self.is_custom_api {
            self.listing.ensure(&model.id).await?;
        }

        self.chat
            .structured(&self.listing, messages, model, option)
            .await
    }

    pub async fn image_generate(
        &self,
        model_id: String,
        prompt: String,
        reference_images: Vec<File>,
        aspect_ratio: AspectRatio,
    ) -> Result<super::ImageGenOutput, Error> {
        if !self.is_custom_api {
            self.listing.ensure(&model_id).await?;
        }

        self.image_gen
            .image_generate(
                &self.listing,
                model_id,
                prompt,
                reference_images,
                aspect_ratio,
            )
            .await
    }

    pub async fn video_generate(
        &self,
        model_id: String,
        prompt: String,
        references: Vec<File>,
        option: VideoGenerationOption,
    ) -> Result<super::VideoGenOutput, Error> {
        if !self.is_custom_api {
            self.video_listing.ensure(&model_id).await?;
        }

        self.video_gen
            .video_generate(&self.video_listing, model_id, prompt, references, option)
            .await
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
        let (content_length, body) = Self::embedding_body(req);
        let mut req_builder = self
            .http_client
            .post(&self.embedding_endpoint)
            .bearer_auth(&self.api_key)
            .header(super::HTTP_REFERER, LLUMEN_URL)
            .header(super::X_TITLE, LLUMEN_NAME)
            .header(CONTENT_TYPE, "application/json");
        if let Some(len) = content_length {
            req_builder = req_builder.header(http::header::CONTENT_LENGTH, len);
        }
        let res = req_builder.body(body).send().await.map_err(Error::Http)?;

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

    fn embedding_body(req: raw::EmbeddingBatchReq) -> (Option<usize>, reqwest::Body) {
        let size = req.size();
        let body = reqwest::Body::wrap_stream(req.into_stream());
        (size, body)
    }
}

pub struct Embedding {
    pub price: f64,
    pub response: Vec<Vec<f32>>,
}
