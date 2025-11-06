use anyhow::Result;
use sea_orm::{DeriveActiveEnum, FromJsonQueryResult, entity::prelude::*};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::models;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MessageKind {
    Hidden = 0,
    User = 1,
    Assistant = 2,
    DeepResearch = 3,
}

/// Chunk kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ChunkKind {
    /// Plain text content
    Text = 0,
    /// File contains metadata of the uploaded file, see [FileHandle]
    File = 7,
    /// Plain text content
    Reasoning = 1,
    /// Tool call request, see [ToolCall]
    ToolCall = 2,
    /// Plain text error result
    Error = 3,
    /// JSON annotations in array
    Annotation = 8,
    /// Reserved for future use
    Report = 4,
    Plan = 5,
    Step = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum ModeKind {
    Normal = 0,
    Search = 1,
    Research = 3,
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

        if config.model_id.contains(":online") {
            return Err("\"online\" suffix are not allowed, see https://openrouter.ai/docs/faq#what-are-model-variants".to_string());
        }
        config.parameter.check().map_err(|x| x.to_owned())?;

        Ok(config)
    }
    pub fn get_config(&self) -> Option<ModelConfig> {
        toml::from_str(&self.config).ok()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default, Serialize)]
pub enum OcrEngine {
    Native,
    Text,
    Mistral,
    #[default]
    Disabled,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
pub struct ModelCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr: Option<OcrEngine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize)]
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
        if let Some(image) = self.capability.image {
            return image;
        }
        models::support_image(&self.model_id)
    }
    pub fn is_audio_capable(&self) -> bool {
        if let Some(audio) = self.capability.audio {
            return audio;
        }
        models::support_audio(&self.model_id)
    }
    pub fn is_other_file_capable(&self) -> bool {
        self.capability.ocr != Some(OcrEngine::Disabled)
    }
    pub fn is_tool_capable(&self) -> bool {
        if let Some(tool) = self.capability.tool {
            return tool;
        }
        models::support_tool(&self.model_id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileHandle {
    pub name: String,
    pub id: i32,
}

/// Status of a deep research step
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum DeepStepStatus {
    InProgress,
    Completed,
    Failed,
}

impl crate::chunk::Model {
    pub fn as_tool_call(&self) -> Result<ToolCall> {
        debug_assert_eq!(self.kind, ChunkKind::ToolCall);
        Ok(serde_json::from_str(&self.content)?)
    }
    pub fn as_file(&self) -> Result<FileHandle> {
        debug_assert_eq!(self.kind, ChunkKind::File);
        Ok(serde_json::from_str(&self.content)?)
    }
}
