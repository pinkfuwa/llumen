use crate::{
    chat::{CompletionContext, PipelineContext, pipeline::chat::PipelineInner},
    openrouter,
};
use anyhow::{Context, Result};

pub struct NormalPipelineInner;

impl PipelineInner for NormalPipelineInner {
    fn get_system_prompt(_: &PipelineContext, _: &CompletionContext) -> Result<String> {
        Ok("You are a helpful assistant.".to_string())
    }

    fn get_model(
        _: &PipelineContext,
        completion_ctx: &CompletionContext,
    ) -> Result<openrouter::Model> {
        let model = completion_ctx
            .model
            .get_config()
            .context("Failed to get model config")?;
        Ok(model.into())
    }
}
