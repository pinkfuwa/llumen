use protocol::{ModelConfig, OcrEngine};
use serde::de::DeserializeOwned;

use crate::openrouter;

impl From<ModelConfig> for openrouter::Model {
    fn from(value: ModelConfig) -> Self {
        openrouter::Model {
            id: value.model_id,
            temperature: value.parameter.temperature,
            repeat_penalty: value.parameter.repeat_penalty,
            top_k: value.parameter.top_k,
            top_p: value.parameter.top_p,
            online: false,
        }
    }
}

pub trait ModelCapability {
    fn is_image_capable(&self) -> bool;
    fn is_audio_capable(&self) -> bool;
    fn is_other_file_capable(&self) -> bool;
    fn is_tool_capable(&self) -> bool;
    fn is_json_capable(&self) -> bool;
}

impl ModelCapability for ModelConfig {
    fn is_image_capable(&self) -> bool {
        if let Some(image) = self.capability.image {
            return image;
        }
        true
    }
    fn is_audio_capable(&self) -> bool {
        if let Some(audio) = self.capability.audio {
            return audio;
        }
        true
    }
    fn is_other_file_capable(&self) -> bool {
        self.capability.ocr != Some(OcrEngine::Disabled)
    }
    fn is_tool_capable(&self) -> bool {
        if let Some(tool) = self.capability.tool {
            return tool;
        }
        true
    }
    fn is_json_capable(&self) -> bool {
        if let Some(json) = self.capability.json {
            return json;
        }
        true
    }
}

pub trait ModelChecker
where
    Self: Sized,
{
    fn check(&self) -> anyhow::Result<()>;
    fn check_config(config: &str) -> anyhow::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let config = toml::from_str::<Self>(config)?;

        config.check()?;

        Ok(config)
    }
    fn from_toml(config: &str) -> anyhow::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let config = toml::from_str::<Self>(config)?;

        config.check()?;

        Ok(config)
    }
}

impl ModelChecker for ModelConfig {
    fn check(&self) -> anyhow::Result<()> {
        if self.model_id.contains(":online") {
            anyhow::bail!(
                "\"online\" suffix are not allowed, see https://openrouter.ai/docs/faq#what-are-model-variants"
            );
        }
        if let Some(temperature) = self.parameter.temperature {
            if temperature < 0.0 || temperature > 1.0 {
                anyhow::bail!("temperature must be between 0.0 and 1.0");
            }
        }
        if let Some(top_p) = self.parameter.top_p {
            if top_p < 0.0 || top_p > 1.0 {
                anyhow::bail!("top_p must be between 0.0 and 1.0");
            }
        }

        if let Some(top_k) = self.parameter.top_k {
            if top_k < 0 || top_k > 100 {
                anyhow::bail!("top_k must be between 0 and 100");
            }
        }

        if let Some(repetition_penalty) = self.parameter.repeat_penalty {
            if repetition_penalty < 1.0 {
                anyhow::bail!("repetition_penalty must be greater than or equal to 1.0");
            }
        }

        Ok(())
    }
}
