use sea_orm::{DeriveActiveEnum, FromJsonQueryResult, entity::prelude::*};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// need patch `Message::Kind`
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MessageKind {
    User = 0,
    Assistant = 1,
    Reasoning = 2,
    System = 3,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[typeshare]
pub struct UserPreference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_on_enter: Option<String>,
}

impl crate::entities::model::Model {
    pub fn check_config(config: &str) -> Result<ModelConfig, String> {
        let config = toml::from_str::<ModelConfig>(config).map_err(|e| e.to_string())?;

        config.parameter.check().map_err(|x| x.to_owned())?;

        Ok(config)
    }
    pub fn get_config(&self) -> Option<ModelConfig> {
        toml::from_str(&self.config).ok()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default, Serialize)]
#[typeshare]
pub enum OcrEngine {
    Native,
    Text,
    Mistral,
    #[default]
    Disabled,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
#[typeshare]
pub struct ModelCapability {
    #[serde(default)]
    pub image: bool,
    #[serde(default)]
    pub audio: bool,
    #[serde(default)]
    pub ocr: OcrEngine,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
#[typeshare]
pub struct ModelParameter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

impl ModelParameter {
    fn check(&self) -> Result<(), &'static str> {
        if let Some(temperature) = self.temperature {
            if temperature < 0.0 || temperature > 1.0 {
                return Err("Temperature must be between 0.0 and 1.0");
            }
        }
        if let Some(repeat_penalty) = self.repeat_penalty {
            if repeat_penalty < 1.0 || repeat_penalty > 2.0 {
                return Err("Repeat penalty must be between 1.0 and 2.0");
            }
        }
        if let Some(top_k) = self.top_k {
            if top_k < 0 || top_k > 100 {
                return Err("Top K must be between 0 and 100");
            }
        }
        if let Some(top_p) = self.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                return Err("Top P must be between 0.0 and 1.0");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[typeshare]
pub struct ModelConfig {
    pub display_name: String,
    pub model_id: String,
    #[serde(default)]
    pub capability: ModelCapability,
    #[serde(default)]
    pub parameter: ModelParameter,
}

impl ModelConfig {
    pub fn is_image_capable(&self) -> bool {
        self.capability.image
    }
    pub fn is_audio_capable(&self) -> bool {
        self.capability.audio
    }
    pub fn is_other_file_capable(&self) -> bool {
        self.capability.ocr != OcrEngine::Disabled
    }
}
