use std::sync::Arc;

use anyhow::Result;
use tokio_stream::StreamExt;

use super::ExecutionStrategy;
use super::processor::StreamProcessor;
use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::chat::{CompletionSession, Context};
use crate::openrouter;

/// Mutable state carried across the streaming loop.
pub struct RunState {
    pub ctx: Arc<Context>,
    pub session: CompletionSession,
    pub model: openrouter::Model,
    pub messages: Vec<openrouter::Message>,
}

/// Runs a strategy from start to finish.
///
/// This is the shared orchestration that ALL modes use:
/// 1. Resolve model + capabilities
/// 2. Prepare execution (build messages, tools, options)
/// 3. Stream tokens from the LLM
/// 4. Handle tool calls, images, annotations
/// 5. Save to database
///
/// The `strategy` argument controls what's different per mode:
/// which prompt template, which tools, how tool calls are handled.
pub async fn run(
    strategy: &dyn ExecutionStrategy,
    ctx: Arc<Context>,
    session: CompletionSession,
) -> Result<()> {
    // 1. Resolve model from stored config
    let model_config = session.get_model_config()?;
    let model: openrouter::Model = model_config.into();
    let capability = ctx.get_capability(&model).await;

    // 2. Prepare execution (strategy builds messages + options)
    let execution = strategy.prepare(&ctx, &session, &capability).await?;

    // 3. Enter the streaming loop
    let mut state = RunState {
        ctx,
        session,
        model,
        messages: execution.messages,
    };

    if let Err(err) = process_loop(&mut state, execution.options, strategy).await {
        state.session.add_error(err.to_string());
    }

    // 4. Save everything to database
    state.session.save().await?;

    Ok(())
}

/// The core streaming loop. Sends messages to the LLM, processes the response,
/// handles tool calls, and recurses if the LLM wants to call more tools.
async fn process_loop(
    state: &mut RunState,
    completion_option: openrouter::CompletionOption,
    strategy: &dyn ExecutionStrategy,
) -> Result<()> {
    let messages = state.messages.clone();

    // Stream from OpenRouter
    let model = openrouter::ModelBuilder::from_model(&state.model).build();
    let mut res: openrouter::StreamCompletion = state
        .ctx
        .openrouter
        .stream(model, messages, completion_option.clone())
        .await?;

    // Pipe LLM tokens -> publisher -> SSE -> browser
    let halt_with_error = state
        .session
        .put_stream((&mut res).map(|resp| resp.map(openrouter_to_buffer_token)))
        .await;

    let result = res.get_result();

    // Post-process: update usage, store chunks, handle images/annotations
    StreamProcessor::process_result(state, &result).await?;

    let halt = halt_with_error?;
    if matches!(halt, StreamEndReason::Halt) {
        log::debug!("The stream was halted");
    }

    // Handle finish reason
    match result.stop_reason {
        openrouter::FinishReason::Stop => {}
        openrouter::FinishReason::Error => log::warn!("cannot capture error"),
        openrouter::FinishReason::Length => log::warn!("The response is too long"),
        openrouter::FinishReason::ToolCalls => {
            if result.toolcalls.is_empty() {
                return Err(anyhow::anyhow!(
                    "No tool calls found, but finish reason is tool_calls"
                ));
            }

            // Delegate tool execution to the mode-specific strategy
            let finalized = strategy.handle_tool_calls(state, result.toolcalls).await?;

            if !finalized {
                // LLM wants to continue after tool results — loop
                return Box::pin(process_loop(state, completion_option, strategy)).await;
            }
        }
    };

    Ok(())
}
