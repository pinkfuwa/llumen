use crate::{
    chat::{CompletionContext, Context, process::chat::ChatInner, prompt::PromptKind},
    openrouter,
};
use anyhow::{Context as _, Result};

pub struct Inner;

impl ChatInner for Inner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String> {
        ctx.prompt
            .render(PromptKind::Normal, completion_ctx)
            .context("Failed to render system prompt")
    }

    fn get_model(_: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model> {
        let model = completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;
        Ok(model.into())
    }
}
