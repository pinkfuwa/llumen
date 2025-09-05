use entity;

use crate::openrouter;

impl From<entity::ModelConfig> for openrouter::Model {
    fn from(value: entity::ModelConfig) -> Self {
        openrouter::Model {
            id: value.model_id,
            temperature: value.parameter.temperature,
            repeat_penalty: value.parameter.repeat_penalty,
            top_k: value.parameter.top_k,
            top_p: value.parameter.top_p,
        }
    }
}
