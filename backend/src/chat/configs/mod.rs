//! available configuration: deep-research(deep)/normal/search
//!
//! each mode are different system prompt with different tool
//!
//! tool can not only handle the request, but also complete take the control of completion
//!
//! For example, deep-research use coordinator to trigger Planner agent...

use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;
use protocol::ModeKind;

use crate::chat::{CompletionContext, Context};

mod configuration;
pub mod deep;
mod executor;
mod normal;
mod search;

pub use configuration::Configuration;

pub struct Configurations {
    normal: Configuration,
    search: Configuration,
    deep: Configuration,
}

impl Configurations {
    pub fn new() -> Self {
        Self {
            normal: normal::normal_configuration(),
            search: search::search_configuration(),
            deep: deep::deep_configuration(),
        }
    }

    pub fn process(
        &self,
        ctx: Arc<Context>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, Result<()>> {
        let mode = completion_ctx.get_mode();
        match mode {
            ModeKind::Normal => self.normal.process(ctx, completion_ctx),
            ModeKind::Search => self.search.process(ctx, completion_ctx),
            ModeKind::Research => self.deep.process(ctx, completion_ctx),
        }
    }
}

impl Default for Configurations {
    fn default() -> Self {
        Self::new()
    }
}
