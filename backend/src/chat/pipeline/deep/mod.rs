pub mod agent;
mod helper;

use anyhow::Result;
use futures_util::future::BoxFuture;

use super::{Pipeline, RunState};
use crate::chat::prompt;
use crate::chat::Context;
use crate::openrouter::{self, Capability, ToolCall};

/// Deep research mode: coordinator hands off to multi-step research agent.
pub struct DeepPipeline;

impl Pipeline for DeepPipeline {
    fn prompt_kind(&self) -> prompt::PromptKind {
        prompt::PromptKind::Coordinator
    }

    fn completion_option(
        &self,
        _ctx: &Context,
        _capability: &Capability,
    ) -> openrouter::CompletionOption {
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

        openrouter::CompletionOption::builder()
            .tools(&[handoff_tool])
            .build()
    }

    /// Deep research doesn't inject context — it builds its own specialized prompts.
    fn inject_context(&self) -> bool {
        false
    }

    fn handle_tool_calls<'a>(
        &'a self,
        state: &'a mut RunState,
        toolcalls: Vec<ToolCall>,
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            agent::DeepAgent::handoff_tool_static(
                &state.ctx,
                &mut state.session,
                toolcalls,
            )
            .await?;
            Ok(true) // Finalized — deep agent handles everything
        })
    }
}
