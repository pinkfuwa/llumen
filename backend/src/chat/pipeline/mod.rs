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
mod execution;
pub mod message_builder;
pub(crate) mod model_strategy;
mod normal;
mod processor;
mod runner;
mod search;

pub use execution::Execution;
pub use runner::RunState;

/// An ExecutionStrategy defines the mode-specific behavior for a chat completion.
///
/// Think of it like a recipe:
/// - `prompt_kind()`: what template to use for the system message
/// - `completion_option()`: what tools and settings to send to the LLM
/// - `inject_context()`: whether to add the context message
/// - `prepare()`: builds the complete Execution (messages + options)
/// - `handle_tool_calls()`: what to do when the LLM calls a tool
///
/// The shared runner (`runner::run`) handles everything else:
/// streaming, images, annotations, database saves.
pub trait ExecutionStrategy: Send + Sync {
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

    /// Prepares the Execution for the LLM call.
    ///
    /// This is a convenience method that:
    /// 1. Renders the system prompt
    /// 2. Converts chat history to OpenRouter format
    /// 3. Builds messages (with or without context)
    /// 4. Gets completion options (tools, temperature, etc)
    /// 5. Returns an Execution ready to send
    ///
    /// **Default implementation**: Uses the other trait methods.
    /// Override if you need custom logic (e.g., Deep research).
    fn prepare<'a>(
        &'a self,
        ctx: &'a Context,
        session: &'a CompletionSession,
        capability: &'a Capability,
    ) -> BoxFuture<'a, Result<Execution>> {
        use crate::chat::converter::db_message_to_openrouter;
        use crate::chat::pipeline::message_builder::MessageBuilder;
        use crate::chat::pipeline::model_strategy;

        Box::pin(async move {
            // 1. Render system prompt
            let system_prompt = ctx.prompt.render(self.prompt_kind(), session)?;

            // 2. Convert chat history from DB format to OpenRouter format
            let mut history = Vec::new();
            for m in &session.messages {
                history.extend(db_message_to_openrouter(ctx, &m.inner).await?);
            }

            // 3. Build message list
            let strategy = model_strategy::get_model_strategy(capability);
            let context_prompt = ctx.prompt.render_context(session)?;

            let messages = if self.inject_context() {
                MessageBuilder::new(system_prompt)
                    .history(history)
                    .context(strategy.as_ref(), context_prompt)
                    .build()
            } else {
                MessageBuilder::new(system_prompt).history(history).build()
            };

            // 4. Get completion options
            let options = self.completion_option(ctx, capability);

            Ok(Execution::new(messages, options))
        })
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
pub struct Strategies {
    normal: Arc<dyn ExecutionStrategy>,
    search: Arc<dyn ExecutionStrategy>,
    deep: Arc<dyn ExecutionStrategy>,
}

impl Strategies {
    pub fn new() -> Self {
        Self {
            normal: Arc::new(normal::NormalStrategy),
            search: Arc::new(search::SearchStrategy),
            deep: Arc::new(deep::DeepStrategy),
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

impl Default for Strategies {
    fn default() -> Self {
        Self::new()
    }
}
