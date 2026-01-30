use std::collections::HashMap;
use std::sync::Arc;

use protocol::OcrEngine;

use crate::openrouter::Capability;

use super::{Error, raw};

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

/// Optimized model capability cache with efficient memory layout
#[derive(Clone)]
pub(super) struct ModelCache {
    pub text_output: bool,
    pub image_output: bool,
    pub image_input: bool,
    pub structured_output: bool,
    pub toolcall: bool,
    pub audio: bool,
    pub reasoning: bool,
    pub support_native_ocr: bool,
}

impl From<ModelCache> for Capability {
    fn from(value: ModelCache) -> Self {
        Capability {
            text_output: value.text_output,
            image_output: value.image_output,
            image_input: value.image_input,
            structured_output: value.structured_output,
            toolcall: value.toolcall,
            ocr: match value.support_native_ocr {
                true => OcrEngine::Native,
                false => OcrEngine::Text,
            },
            audio: value.audio,
            reasoning: value.reasoning,
        }
    }
}

impl From<&raw::Model> for ModelCache {
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

impl ModelCache {
    pub fn image_only() -> Self {
        Self {
            text_output: false,
            image_output: true,
            image_input: false,
            structured_output: false,
            toolcall: false,
            audio: false,
            reasoning: false,
            support_native_ocr: false,
        }
    }
}

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

pub(super) struct ModelCacheManager {
    models: Arc<tokio::sync::RwLock<HashMap<String, ModelCache>>>,
    fetch_mutex: Arc<tokio::sync::Mutex<()>>,
    models_endpoint: String,
    api_key: String,
}

impl ModelCacheManager {
    pub fn new(models_endpoint: String, api_key: String) -> Self {
        let models = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let fetch_mutex = Arc::new(tokio::sync::Mutex::new(()));

        {
            let models_clone = models.clone();
            let api_key_clone = api_key.clone();
            let endpoint_clone = models_endpoint.clone();
            tokio::spawn(async move {
                match fetch_models(&endpoint_clone, &api_key_clone).await {
                    Ok(model_list) => {
                        log::info!("{} models available", model_list.len());
                        let map: HashMap<String, ModelCache> = model_list
                            .iter()
                            .map(|m| (m.id.clone(), ModelCache::from(m)))
                            .collect();
                        *models_clone.write().await = map;
                    }
                    Err(err) => log::error!("Failed to fetch models: {}", err),
                }
            });
        }

        Self {
            models,
            fetch_mutex,
            models_endpoint,
            api_key,
        }
    }

    pub async fn get_model_ids(&self) -> Vec<String> {
        self.models.read().await.keys().cloned().collect()
    }

    pub async fn get(&self, model_id: &str) -> Option<ModelCache> {
        self.models.read().await.get(model_id).cloned()
    }

    /// Ensure a model is in cache. Fetch the model list if missing.
    /// Uses a mutex to ensure only one fetch happens at a time.
    pub async fn ensure_model(&self, model_id: &str) -> Result<(), Error> {
        let model_id = model_id.split(':').next().unwrap();

        // Fast path: check if model already exists
        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        // Acquire mutex to ensure only one fetch at a time
        let _guard = self.fetch_mutex.lock().await;

        // Double-check after acquiring lock (another thread may have fetched)
        {
            let models = self.models.read().await;
            if models.contains_key(model_id) {
                return Ok(());
            }
        }

        // Fetch the full model list
        log::info!("Model {} not in cache, fetching full model list", model_id);
        let model_list = fetch_models(&self.models_endpoint, &self.api_key).await?;

        log::info!("Fetched {} models", model_list.len());

        let mut new_models: HashMap<String, ModelCache> = model_list
            .iter()
            .map(|m| (m.id.clone(), ModelCache::from(m)))
            .collect();

        // Check if the requested model was in the fetched list
        if !new_models.contains_key(model_id) {
            log::warn!(
                "Model {} not found in fetched list, treating as image-only model",
                model_id
            );
            new_models.insert(model_id.to_string(), ModelCache::image_only());
        }

        *self.models.write().await = new_models;
        Ok(())
    }
}
