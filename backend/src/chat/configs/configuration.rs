use std::sync::Arc;

use anyhow::Result;
use futures_util::future::BoxFuture;
use tokio_stream::StreamExt;

use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::utils::model::ModelChecker;
use crate::{chat::*, openrouter};
use entity::file;
use protocol::ModelConfig;
use sea_orm::ActiveValue;

#[derive(Clone)]
pub struct Configuration {
    pub completion_option: openrouter::CompletionOption,
    pub tool_handler: Arc<
        dyn for<'a> Fn(
                &'a mut ProcessState,
                Vec<openrouter::ToolCall>,
            ) -> BoxFuture<'a, Result<bool, anyhow::Error>>
            + Send
            + Sync,
    >,
    pub prompt: prompt::PromptKind,
}

pub struct ProcessState {
    pub ctx: Arc<Context>,
    pub completion_ctx: CompletionContext,
    pub model: openrouter::Model,
    pub messages: Vec<openrouter::Message>,
}

impl Configuration {
    pub fn process(
        &self,
        ctx: Arc<Context>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, Result<()>> {
        let prompt = self.prompt;
        let completion_option = self.completion_option.clone();
        let tool_handler = self.tool_handler.clone();

        Box::pin(async move {
            let model = <ModelConfig as ModelChecker>::from_toml(&completion_ctx.model.config)
                .expect("Failed to get model config");
            let model = model.into();
            let system_prompt = ctx.prompt.render(prompt, &completion_ctx)?;

            let mut messages = vec![openrouter::Message::System(system_prompt)];

            for m in &completion_ctx.messages {
                messages.extend(db_message_to_openrouter(&ctx, &m.inner).await?);
            }

            let mut state = ProcessState {
                ctx,
                completion_ctx,
                model,
                messages,
            };

            if let Some(err) = Self::process_loop(&mut state, completion_option, tool_handler)
                .await
                .err()
            {
                state.completion_ctx.add_error(err.to_string());
            }
            state.completion_ctx.save().await?;

            Ok(())
        })
    }

    pub async fn process_loop(
        state: &mut ProcessState,
        completion_option: openrouter::CompletionOption,
        tool_handler: Arc<
            dyn for<'a> Fn(
                    &'a mut ProcessState,
                    Vec<openrouter::ToolCall>,
                ) -> BoxFuture<'a, Result<bool, anyhow::Error>>
                + Send
                + Sync,
        >,
    ) -> Result<(), anyhow::Error> {
        let message = state.messages.clone();

        let model = openrouter::ModelBuilder::from_model(&state.model).build();
        let mut res: openrouter::StreamCompletion = state
            .ctx
            .openrouter
            .stream(model, message, completion_option.clone())
            .await?;

        let halt_with_error = state
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(openrouter_to_buffer_token)))
            .await;

        let result = res.get_result();

        state
            .completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        state
            .completion_ctx
            .message
            .inner
            .as_assistant()
            .unwrap()
            .extend(openrouter_stream_to_assitant_chunk(&result.responses));

        if let Some(reasoning_details) = result.reasoning_details {
            state
                .completion_ctx
                .message
                .inner
                .add_reasoning_detail(reasoning_details);
        }

        if !result.image.is_empty() {
            for image in &result.image {
                let chat_id = state.completion_ctx.get_chat_id();
                let mime_type = image.mime_type.clone();
                let owner_id = state.completion_ctx.user.id;
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

                state.completion_ctx.message.inner.add_image(file_id);
                state.completion_ctx.add_token(Token::Image(file_id));
            }
        }

        let halt = halt_with_error?;
        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let mut finalized = true;
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

                finalized = tool_handler(state, result.toolcalls).await?;
                if let Some(annotations) = result.annotations {
                    state
                        .completion_ctx
                        .message
                        .inner
                        .add_annotation(annotations);
                }
            }
        };

        if finalized {
            return Ok(());
        }
        Box::pin(Self::process_loop(state, completion_option, tool_handler)).await
    }
}
