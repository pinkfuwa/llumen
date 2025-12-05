use std::sync::Arc;

use super::configuration::Configuration;
use crate::{chat::prompt, openrouter};

pub fn normal_configuration() -> Configuration {
    Configuration {
        completion_option: openrouter::CompletionOption::default(),
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
