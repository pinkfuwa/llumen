pub mod agent;
mod helper;

use std::sync::Arc;

use super::builder::ConfigurationBuilder;
use super::configuration::Configuration;
use crate::{chat::*, openrouter};

pub fn deep_configuration() -> Configuration {
    let handoff_tool = openrouter::Tool {
        name: "handoff_to_planner".to_string(),
        description: "Handoff to planner agent to do plan.".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {
                "research_topic": {
                    "type": "string",
                    "description": "The topic of the research task to be handed off."
                },
                "locale": {
                    "type": "string",
                    "description": "The user's detected language locale (e.g., en-US, zh-TW)."
                }
            },
            "required": ["research_topic", "locale"]
        }),
    };

    ConfigurationBuilder::new(prompt::PromptKind::Coordinator)
        .with_tool(handoff_tool)
        .with_handler(Arc::new(|state, toolcalls| {
            Box::pin(async move {
                agent::DeepAgent::handoff_tool_static(
                    &state.ctx,
                    &mut state.completion_ctx,
                    toolcalls,
                )
                .await?;
                Ok(true)
            })
        }))
        .build()
}
