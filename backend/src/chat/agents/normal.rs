use crate::{
    chat::{CompletionContext, Context, agent::chat::ChatInner, prompt::PromptKind},
    openrouter,
    utils::model::ModelChecker,
};
use anyhow::{Context as _, Result};
use protocol::ModelConfig;

pub struct Inner;

impl ChatInner for Inner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String> {
        ctx.prompt
            .render(PromptKind::Normal, completion_ctx)
            .context("Failed to render system prompt")
    }

    fn get_model(_: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model> {
        let model = <ModelConfig as ModelChecker>::from_toml(&completion_ctx.model.config)
            .context("Failed to get model config")?;
        Ok(model.into())
    }
}
