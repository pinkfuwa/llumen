use std::sync::Arc;

use futures_util::future::BoxFuture;

use super::{CompletionContext, PipelineContext};

pub mod chat;
pub mod helper;

pub trait Pipeline {
    fn process(
        ctx: Arc<PipelineContext>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, anyhow::Result<()>>;
}
