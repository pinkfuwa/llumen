//! Completion strategies dispatched by [`super::Context`].
//!
//! Each chat mode (Normal, Search, DeepResearch) is implemented as a
//! standalone function that receives the shared context and a mutable
//! session.  The strategy module re-exports the [`Strategy`] enum used by
//! callers to select which mode to run.

mod normal;
mod search;

use super::session::CompletionSession;
use super::context::Context;

use std::sync::Arc;

/// Selects which completion strategy to execute.
#[derive(Debug, Clone)]
pub enum Strategy {
    Normal,
    Search,
    DeepResearch,
}

impl From<crate::utils::chat::ChatMode> for Strategy {
    fn from(mode: crate::utils::chat::ChatMode) -> Self {
        match mode {
            crate::utils::chat::ChatMode::Normal => Self::Normal,
            crate::utils::chat::ChatMode::Search => Self::Search,
            crate::utils::chat::ChatMode::Research => Self::DeepResearch,
        }
    }
}

/// Dispatches a strategy, running the appropriate completion pipeline.
pub async fn dispatch(
    ctx: Arc<Context>,
    strategy: Strategy,
    session: &mut CompletionSession,
) -> anyhow::Result<()> {
    match strategy {
        Strategy::Normal => normal::execute(&ctx, session).await,
        Strategy::Search => search::execute(&ctx, session).await,
        Strategy::DeepResearch => {
            super::deep_research::DeepAgent::handoff_tool_static(&ctx, session, Vec::new()).await
        }
    }
}
