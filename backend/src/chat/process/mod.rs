use std::sync::Arc;

use futures_util::future::BoxFuture;

use super::{CompletionContext, Context};

pub mod chat;
pub mod helper;

pub trait Pipeline {
    fn process(
        ctx: Arc<Context>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, anyhow::Result<()>>;
}
