//! The pipeline module orchestrates how each chat mode processes a completion.
//!
//! Architecture overview:
//!
//! ```text
//!   Route handler
//!       │
//!       ▼
//!   Pipelines::process()          ← picks the right pipeline for the mode
//!       │
//!       ▼
//!   runner::run(pipeline, ...)    ← shared orchestration (message building,
//!       │                           streaming, image saving, DB save)
//!       │
//!       ├─ pipeline.prompt_kind()         ← which template?
//!       ├─ pipeline.completion_option()   ← which tools? what settings?
//!       ├─ pipeline.inject_context()      ← add context message?
//!       ├─ MessageBuilder                 ← pure message list construction
//!       └─ pipeline.handle_tool_calls()   ← mode-specific tool execution
//! ```
//!
//! Each mode (normal, search, deep research) implements the `Pipeline` trait.
//! The shared runner handles everything that's the same across modes:
//! streaming, image saving, annotation handling, database persistence.

use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;
use protocol::ModeKind;

use crate::chat::prompt;
use crate::chat::{CompletionSession, Context};
use crate::openrouter::{self, Capability, ToolCall};

pub mod deep;
pub mod message_builder;
pub(crate) mod model_strategy;
mod normal;
mod runner;
mod search;

pub use runner::RunState;

/// A Pipeline defines the mode-specific behavior for a chat completion.
///
/// Think of it like a recipe:
/// - `prompt_kind()`: what template to use for the system message
/// - `completion_option()`: what tools and settings to send to the LLM
/// - `inject_context()`: whether to add the context message
/// - `handle_tool_calls()`: what to do when the LLM calls a tool
///
/// The shared runner (`runner::run`) handles everything else:
/// streaming, images, annotations, database saves.
pub trait Pipeline: Send + Sync {
    /// Which prompt template this mode uses (Normal, Search, Coordinator, etc.)
    fn prompt_kind(&self) -> prompt::PromptKind;

    /// Build the CompletionOption for this request.
    ///
    /// This is where each mode configures its tools, temperature, etc.
    /// The `ctx` is provided so you can check things like compatibility mode.
    /// The `capability` is provided so you can filter tools for the model.
    fn completion_option(
        &self,
        ctx: &Context,
        capability: &Capability,
    ) -> openrouter::CompletionOption;

    /// Whether to inject the context message after chat history.
    ///
    /// Most modes return `true`. Deep research returns `false` because
    /// it uses its own specialized prompts.
    ///
    /// Note: even when this returns `true`, the MessageBuilder will still
    /// skip context for image-only models (handled by ModelStrategy).
    fn inject_context(&self) -> bool {
        true
    }

    /// Handle tool calls from the LLM response.
    ///
    /// Returns `true` if the completion is done (no more streaming needed).
    /// Returns `false` if the LLM should continue (tool results added to messages).
    fn handle_tool_calls<'a>(
        &'a self,
        state: &'a mut RunState,
        calls: Vec<ToolCall>,
    ) -> BoxFuture<'a, Result<bool>>;
}

/// Routes incoming completions to the right pipeline based on mode.
pub struct Pipelines {
    normal: Arc<dyn Pipeline>,
    search: Arc<dyn Pipeline>,
    deep: Arc<dyn Pipeline>,
}

impl Pipelines {
    pub fn new() -> Self {
        Self {
            normal: Arc::new(normal::NormalPipeline),
            search: Arc::new(search::SearchPipeline),
            deep: Arc::new(deep::DeepPipeline),
        }
    }

    /// Process a completion request by routing to the appropriate pipeline.
    pub fn process(
        &self,
        ctx: Arc<Context>,
        completion_ctx: CompletionSession,
    ) -> BoxFuture<'static, Result<()>> {
        let mode = completion_ctx.get_mode();
        let pipeline = match mode {
            ModeKind::Normal => self.normal.clone(),
            ModeKind::Search => self.search.clone(),
            ModeKind::Research => self.deep.clone(),
        };

        Box::pin(async move { runner::run(pipeline.as_ref(), ctx, completion_ctx).await })
    }
}

impl Default for Pipelines {
    fn default() -> Self {
        Self::new()
    }
}
