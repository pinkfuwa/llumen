use std::sync::Arc;

use anyhow::Result;
use tokio_stream::StreamExt;

use super::message_builder::MessageBuilder;
use super::model_strategy;
use super::Pipeline;
use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::chat::token::Token;
use crate::chat::{CompletionContext, Context};
use crate::openrouter;
use entity::file;
use sea_orm::ActiveValue;

/// Mutable state carried across the streaming loop.
///
/// If you're reading this to understand the architecture:
/// - `ctx`      = shared services (DB, openrouter, blob storage, tools)
/// - `session`  = per-request state (publisher, message being built, usage)
/// - `model`    = resolved model config for this request
/// - `messages`  = the growing message list sent to the LLM
pub struct RunState {
    pub ctx: Arc<Context>,
    pub session: CompletionContext,
    pub model: openrouter::Model,
    pub messages: Vec<openrouter::Message>,
}

/// Runs a pipeline from start to finish.
///
/// This is the shared orchestration that ALL modes use:
/// 1. Resolve model + capabilities
/// 2. Build the message list (system prompt + history + context)
/// 3. Stream tokens from the LLM
/// 4. Handle tool calls, images, annotations
/// 5. Save to database
///
/// The `pipeline` argument controls what's different per mode:
/// which prompt template, which tools, how tool calls are handled.
pub async fn run(
    pipeline: &dyn Pipeline,
    ctx: Arc<Context>,
    session: CompletionContext,
) -> Result<()> {
    // 1. Resolve model from stored config
    let model_config = session.get_model_config()?;
    let model: openrouter::Model = model_config.into();
    let capability = ctx.get_capability(&model).await;

    // 2. Build completion options (tools, temperature, etc.)
    let completion_option = pipeline.completion_option(&ctx, &capability);

    // 3. Render system prompt
    let system_prompt = ctx.prompt.render(pipeline.prompt_kind(), &session)?;

    // 4. Convert chat history from DB format to OpenRouter format
    let mut history = Vec::new();
    for m in &session.messages {
        history.extend(db_message_to_openrouter(&ctx, &m.inner).await?);
    }

    // 5. Build message list (MessageBuilder handles context injection via strategy)
    let strategy = model_strategy::get_model_strategy(&capability);
    let context_prompt = ctx.prompt.render_context(&session)?;

    let messages = if pipeline.inject_context() {
        MessageBuilder::new(system_prompt)
            .history(history)
            .context(strategy.as_ref(), context_prompt)
            .build()
    } else {
        MessageBuilder::new(system_prompt).history(history).build()
    };

    // 6. Enter the streaming loop
    let mut state = RunState {
        ctx,
        session,
        model,
        messages,
    };

    if let Err(err) = process_loop(&mut state, completion_option, pipeline).await {
        state.session.add_error(err.to_string());
    }

    // 7. Save everything to database
    state.session.save().await?;

    Ok(())
}

/// The core streaming loop. Sends messages to the LLM, processes the response,
/// handles tool calls, and recurses if the LLM wants to call more tools.
async fn process_loop(
    state: &mut RunState,
    completion_option: openrouter::CompletionOption,
    pipeline: &dyn Pipeline,
) -> Result<()> {
    let messages = state.messages.clone();

    // Stream from OpenRouter
    let model = openrouter::ModelBuilder::from_model(&state.model).build();
    let mut res: openrouter::StreamCompletion = state
        .ctx
        .openrouter
        .stream(model, messages, completion_option.clone())
        .await?;

    // Pipe LLM tokens → publisher → SSE → browser
    let halt_with_error = state
        .session
        .put_stream((&mut res).map(|resp| resp.map(openrouter_to_buffer_token)))
        .await;

    let result = res.get_result();

    // Track cost and token usage
    state
        .session
        .update_usage(result.usage.cost as f32, result.usage.token as i32);

    // Store assistant text chunks on the message entity
    state
        .session
        .message
        .inner
        .as_assistant()
        .unwrap()
        .extend(openrouter_stream_to_assitant_chunk(&result.responses));

    // Store reasoning details if present
    if let Some(reasoning_details) = result.reasoning_details {
        state
            .session
            .message
            .inner
            .add_reasoning_detail(reasoning_details);
    }

    // Handle generated images: save to blob storage + DB
    if !result.image.is_empty() {
        for image in &result.image {
            let chat_id = state.session.get_chat_id();
            let mime_type = image.mime_type.clone();
            let owner_id = state.session.user.id;
            let data = image.data.clone();
            let blob = state.ctx.blob.clone();
            let db = &state.ctx.db;

            use sea_orm::ActiveModelTrait;
            let result = file::ActiveModel {
                chat_id: ActiveValue::Set(Some(chat_id)),
                owner_id: ActiveValue::Set(Some(owner_id)),
                mime_type: ActiveValue::Set(Some(mime_type)),
                ..Default::default()
            }
            .insert(db)
            .await?;

            let file_id = result.id;

            let size = data.len();
            if let Err(e) = blob
                .insert(file_id, size, tokio_stream::once(bytes::Bytes::from(data)))
                .await
            {
                log::error!("Failed to store image in blob: {}", e);
                continue;
            }

            state.session.message.inner.add_image(file_id);
            state.session.add_token(Token::Image(file_id));
        }
    }

    // Handle URL citations from annotations
    if let Some(annotations) = result.annotations {
        let citations = openrouter::extract_url_citations(&annotations);
        state.session.message.inner.add_annotation(annotations);
        if !citations.is_empty() {
            state
                .session
                .message
                .inner
                .add_url_citation(citations.clone());
            state.session.add_token(Token::UrlCitation(citations));
        }
    }

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

            // Delegate tool execution to the mode-specific pipeline
            let finalized = pipeline.handle_tool_calls(state, result.toolcalls).await?;

            if !finalized {
                // LLM wants to continue after tool results — loop
                return Box::pin(process_loop(state, completion_option, pipeline)).await;
            }
        }
    };

    Ok(())
}
