use anyhow::Result;
use futures_util::future::BoxFuture;

use super::{Pipeline, RunState};
use crate::chat::prompt;
use crate::openrouter::{self, Capability, ToolCall};
use crate::chat::Context;

/// Normal chat: no tools, supports image generation.
pub struct NormalPipeline;

impl Pipeline for NormalPipeline {
    fn prompt_kind(&self) -> prompt::PromptKind {
        prompt::PromptKind::Normal
    }

    fn completion_option(
        &self,
        _ctx: &Context,
        _capability: &Capability,
    ) -> openrouter::CompletionOption {
        openrouter::CompletionOption::builder()
            .image_generation(true)
            .build()
    }

    fn handle_tool_calls<'a>(
        &'a self,
        _state: &'a mut RunState,
        _calls: Vec<ToolCall>,
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async {
            Err(anyhow::anyhow!(
                "Tool calls are not supported in normal mode"
            ))
        })
    }
}
