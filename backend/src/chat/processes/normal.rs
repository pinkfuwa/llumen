use crate::{
    chat::{CompletionContext, Context, process::chat::ChatInner},
    openrouter,
};
use anyhow::{Context as _, Result};

pub struct NormalPipelineInner;

impl ChatInner for NormalPipelineInner {
    fn get_system_prompt(_: &Context, _: &CompletionContext) -> Result<String> {
        Ok("You are a helpful assistant.".to_string())
    }

    fn get_model(_: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model> {
        let model = completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;
        Ok(model.into())
    }
}
