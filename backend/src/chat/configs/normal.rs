use std::sync::Arc;

use super::builder::ConfigurationBuilder;
use super::configuration::Configuration;
use crate::chat::prompt;

pub fn normal_configuration() -> Configuration {
    ConfigurationBuilder::new(prompt::PromptKind::Normal)
        .with_image_generation(true)
        .with_handler(Arc::new(|_, _| {
            Box::pin(async move {
                Err(anyhow::anyhow!(
                    "Tool calls are not supported in normal mode"
                ))
            })
        }))
        .build()
}
