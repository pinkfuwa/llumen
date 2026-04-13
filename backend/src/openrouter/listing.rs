use std::collections::HashMap;
use std::sync::Arc;

use protocol::{OcrEngine, ReasoningEffort};

use crate::openrouter::Capability;

use super::{raw, Error, HTTP_REFERER, X_TITLE};

#[derive(Clone)]
pub(super) struct ModelCaps {
    pub text_output: bool,
    pub image_output: bool,
    pub image_input: bool,
    pub video_input: bool,
    pub structured_output: bool,
    pub toolcall: bool,
    pub audio: bool,
    pub reasoning: bool,
    pub support_native_ocr: bool,
}

impl From<ModelCaps> for Capability {
    fn from(value: ModelCaps) -> Self {
        Capability {
            text_output: value.text_output,
            image_output: value.image_output,
            image_input: value.image_input,
            video_input: value.video_input,
            structured_output: value.structured_output,
            toolcall: value.toolcall,
            ocr: if value.support_native_ocr {
                OcrEngine::Native
            } else {
                OcrEngine::Text
            },
            audio: value.audio,
            reasoning: value.reasoning,
            reasoning_effort: ReasoningEffort::Auto,
        }
    }
}

impl From<&raw::Model> for ModelCaps {
    fn from(model: &raw::Model) -> Self {
        Self {
            text_output: model
                .architecture
                .output_modalities
                .contains(&raw::Modality::Text),
            image_output: model
                .architecture
                .output_modalities
                .contains(&raw::Modality::Image),
            image_input: model
                .architecture
                .input_modalities
                .contains(&raw::Modality::Image),
            video_input: model
                .architecture
                .input_modalities
                .contains(&raw::Modality::Video),
            structured_output: model
                .supported_parameters
                .contains(&raw::SupportedParams::StructuredOutput),
            toolcall: model
                .supported_parameters
                .contains(&raw::SupportedParams::Tools),
            audio: model
                .architecture
                .input_modalities
                .contains(&raw::Modality::Audio),
            reasoning: model
                .supported_parameters
                .contains(&raw::SupportedParams::Reasoning),
            support_native_ocr: model
                .architecture
                .input_modalities
                .contains(&raw::Modality::File),
        }
    }
}

impl ModelCaps {}

#[derive(Clone)]
pub struct VideoModelCaps {
    pub supported_resolutions: Vec<String>,
    pub supported_aspect_ratios: Vec<String>,
    pub supported_sizes: Vec<String>,
    pub allowed_passthrough_parameters: Vec<String>,
    pub max_reference_images: usize,
    pub max_reference_videos: usize,
    pub supports_generate_audio: bool,
}

impl VideoModelCaps {
    pub fn supports_reference_images(&self) -> bool {
        self.max_reference_images > 0
    }

    pub fn supports_reference_videos(&self) -> bool {
        self.max_reference_videos > 0
    }
}

#[derive(Clone, Copy)]
struct VideoCapabilityFallback {
    max_reference_images: usize,
    max_reference_videos: usize,
    supports_generate_audio: bool,
}

impl VideoCapabilityFallback {
    fn default_unknown() -> Self {
        Self {
            max_reference_images: 0,
            max_reference_videos: 0,
            supports_generate_audio: false,
        }
    }
}

fn hardcoded_video_fallback(model_id: &str) -> VideoCapabilityFallback {
    // OpenRouter video model metadata does not currently guarantee reference
    // count fields; keep deterministic fallbacks for known providers.
    match model_id {
        "google/veo-3.1" | "google/veo-3" | "google/veo-2" => VideoCapabilityFallback {
            max_reference_images: 1,
            max_reference_videos: 0,
            supports_generate_audio: true,
        },
        _ => VideoCapabilityFallback::default_unknown(),
    }
}

fn read_bool(extra: &HashMap<String, serde_json::Value>, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| extra.get(*key).and_then(serde_json::Value::as_bool))
}

fn read_usize(extra: &HashMap<String, serde_json::Value>, keys: &[&str]) -> Option<usize> {
    keys.iter().find_map(|key| {
        extra
            .get(*key)
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
    })
}

impl From<&raw::VideoModel> for VideoModelCaps {
    fn from(model: &raw::VideoModel) -> Self {
        let fallback = hardcoded_video_fallback(&model.id);
        let max_reference_images = read_usize(
            &model.extra,
            &[
                "max_reference_images",
                "max_image_references",
                "image_reference_limit",
            ],
        )
        .unwrap_or(fallback.max_reference_images);
        let max_reference_videos = read_usize(
            &model.extra,
            &[
                "max_reference_videos",
                "max_video_references",
                "video_reference_limit",
            ],
        )
        .unwrap_or(fallback.max_reference_videos);

        let supports_generate_audio = read_bool(
            &model.extra,
            &["supports_generate_audio", "generate_audio", "audio_output"],
        )
        .unwrap_or(fallback.supports_generate_audio);

        Self {
            supported_resolutions: model.supported_resolutions.clone(),
            supported_aspect_ratios: model.supported_aspect_ratios.clone(),
            supported_sizes: model.supported_sizes.clone(),
            allowed_passthrough_parameters: model.allowed_passthrough_parameters.clone(),
            max_reference_images,
            max_reference_videos,
            supports_generate_audio,
        }
    }
}

pub(super) struct ModelListing {
    models: tokio::sync::RwLock<HashMap<String, ModelCaps>>,
    fetch_mutex: tokio::sync::Mutex<()>,
    http_client: reqwest::Client,
    models_endpoint: String,
    api_key: String,
}

pub(super) struct VideoModelListing {
    models: tokio::sync::RwLock<HashMap<String, VideoModelCaps>>,
    fetch_mutex: tokio::sync::Mutex<()>,
    http_client: reqwest::Client,
    models_endpoint: String,
    api_key: String,
}

impl ModelListing {
    pub fn new(
        http_client: reqwest::Client,
        models_endpoint: String,
        api_key: String,
    ) -> Arc<Self> {
        let listing = Arc::new(Self {
            models: tokio::sync::RwLock::new(HashMap::new()),
            fetch_mutex: tokio::sync::Mutex::new(()),
            http_client,
            models_endpoint,
            api_key,
        });

        let listing_clone = listing.clone();
        tokio::spawn(async move {
            match listing_clone.fetch_models().await {
                Ok(model_list) => {
                    log::info!("{} models available", model_list.len());
                    let map: HashMap<String, ModelCaps> = model_list
                        .iter()
                        .map(|model| (model.id.clone(), ModelCaps::from(model)))
                        .collect();
                    *listing_clone.models.write().await = map;
                }
                Err(err) => log::error!("Failed to fetch models: {}", err),
            }
        });

        listing
    }

    pub async fn get_model_ids(&self) -> Vec<String> {
        self.models.read().await.keys().cloned().collect()
    }

    pub async fn get(&self, model_id: &str) -> Option<ModelCaps> {
        self.models.read().await.get(model_id).cloned()
    }

    pub async fn ensure(&self, model_id: &str) -> Result<(), Error> {
        let model_id = model_id.split(':').next().unwrap_or(model_id);

        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        let _guard = self.fetch_mutex.lock().await;

        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        log::info!("Model {} not in cache, fetching full model list", model_id);
        let model_list = self.fetch_models().await?;

        log::info!("Fetched {} models", model_list.len());

        let new_models: HashMap<String, ModelCaps> = model_list
            .iter()
            .map(|model| (model.id.clone(), ModelCaps::from(model)))
            .collect();

        *self.models.write().await = new_models;
        Ok(())
    }

    async fn fetch_models(&self) -> Result<Vec<raw::Model>, Error> {
        let response = self
            .http_client
            .get(&self.models_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .send()
            .await?;

        let body = response.text().await?;
        match serde_json::from_str::<raw::ModelListResponse>(&body) {
            Ok(model_list) => Ok(model_list.data),
            Err(err) => {
                log::warn!("invalid model list: {}", body);
                Err(err.into())
            }
        }
    }
}

impl VideoModelListing {
    pub fn new(
        http_client: reqwest::Client,
        models_endpoint: String,
        api_key: String,
    ) -> Arc<Self> {
        let listing = Arc::new(Self {
            models: tokio::sync::RwLock::new(HashMap::new()),
            fetch_mutex: tokio::sync::Mutex::new(()),
            http_client,
            models_endpoint,
            api_key,
        });

        let listing_clone = listing.clone();
        tokio::spawn(async move {
            match listing_clone.fetch_models().await {
                Ok(model_list) => {
                    log::info!("{} video models available", model_list.len());
                    let map: HashMap<String, VideoModelCaps> = model_list
                        .iter()
                        .map(|model| (model.id.clone(), VideoModelCaps::from(model)))
                        .collect();
                    *listing_clone.models.write().await = map;
                }
                Err(err) => log::error!("Failed to fetch video models: {}", err),
            }
        });

        listing
    }

    pub async fn get_model_ids(&self) -> Vec<String> {
        self.models.read().await.keys().cloned().collect()
    }

    pub async fn get(&self, model_id: &str) -> Option<VideoModelCaps> {
        self.models.read().await.get(model_id).cloned()
    }

    pub async fn ensure(&self, model_id: &str) -> Result<(), Error> {
        let model_id = model_id.split(':').next().unwrap_or(model_id);

        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        let _guard = self.fetch_mutex.lock().await;

        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        log::info!(
            "Video model {} not in cache, fetching full video model list",
            model_id
        );
        let model_list = self.fetch_models().await?;
        log::info!("Fetched {} video models", model_list.len());

        let new_models: HashMap<String, VideoModelCaps> = model_list
            .iter()
            .map(|model| (model.id.clone(), VideoModelCaps::from(model)))
            .collect();

        *self.models.write().await = new_models;
        Ok(())
    }

    async fn fetch_models(&self) -> Result<Vec<raw::VideoModel>, Error> {
        let response = self
            .http_client
            .get(&self.models_endpoint)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", HTTP_REFERER)
            .header("X-Title", X_TITLE)
            .send()
            .await?;

        let body = response.text().await?;
        match serde_json::from_str::<raw::VideoModelListResponse>(&body) {
            Ok(model_list) => Ok(model_list.data),
            Err(err) => {
                log::warn!("invalid video model list: {}", body);
                Err(err.into())
            }
        }
    }
}
