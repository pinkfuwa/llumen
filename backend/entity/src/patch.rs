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
        toml::from_str::<ModelConfig>(config).map_err(|e| e.to_string())
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[typeshare]
pub struct ModelConfig {
    pub display_name: String,
    pub openrouter_id: String,
    #[serde(default)]
    pub capability: ModelCapability,
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
