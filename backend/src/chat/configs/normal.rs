use std::sync::Arc;

use super::configuration::Configuration;
use crate::chat::prompt;

pub fn normal_configuration() -> Configuration {
    Configuration {
        tool: vec![],
        model_setup: Arc::new(|completion_ctx| {
            use crate::utils::model::ModelChecker;
            use protocol::ModelConfig;

            let model = <ModelConfig as ModelChecker>::from_toml(&completion_ctx.model.config)
                .expect("Failed to get model config");
            model.into()
        }),
        tool_handler: Arc::new(|_, _| {
            Box::pin(async move {
                Err(anyhow::anyhow!(
                    "Tool calls are not supported in normal mode"
                ))
            })
        }),
        prompt: prompt::PromptKind::Normal,
    }
}
