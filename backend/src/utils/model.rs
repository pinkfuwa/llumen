use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    pub model: ModelConfigModel,
    pub capability: ModelConfigCapability,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfigModel {
    pub name: String,
    pub model: String,
    pub ocr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfigCapability {
    pub image: bool,
    pub audio: bool,
    pub ocr: bool,
}

pub fn parse(s: &str) -> Result<ModelConfig> {
    toml::from_str(s).context("Cannot parse model config")
}
