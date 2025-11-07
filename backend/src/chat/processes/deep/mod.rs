mod agent;
mod helper;

use anyhow::{Context as _, Result};
use futures_util::future::BoxFuture;

use crate::chat::process::chat::ChatPipeline;
use crate::chat::{CompletionContext, Context};
use crate::openrouter;

use crate::chat::{process::chat::ChatInner, prompt::PromptKind};

pub struct Inner;

impl ChatInner for Inner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String> {
        ctx.prompt
            .render(PromptKind::Coordinator, completion_ctx)
            .context("Failed to render system prompt")
    }

    fn get_model(_: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model> {
        let model = completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;
        let model: openrouter::Model = model.into();

        Ok(model)
    }

    fn get_tools(
        ctx: &Context,
        completion_ctx: &CompletionContext,
    ) -> Result<Vec<openrouter::Tool>> {
        Ok(vec![openrouter::Tool {
            name: "handoff_to_planner".to_string(),
            description: "Handoff to planner agent to do plan.".to_string(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "research_topic": {
                        "type": "string",
                        "description": "The topic of the research task to be handed off."
                    },
                    "local": {
                        "type": "string",
                        "description": "The user's detected language locale (e.g., en-US, zh-TW)."
                    }
                },
                "required": ["research_topic", "local"]
            }),
        }])
    }

    fn handoff_tool<'a>(
        pipeline: &'a mut ChatPipeline<Self>,
        toolcall: Vec<openrouter::ToolCall>,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>>
    where
        Self: Sized,
    {
        agent::DeepAgent::handoff_tool(pipeline, toolcall)
    }
}
