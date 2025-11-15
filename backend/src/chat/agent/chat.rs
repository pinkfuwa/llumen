use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::{Context as _, Result};

use futures_util::future::BoxFuture;
use tokio_stream::StreamExt;

use crate::chat::context::CompletionContext;
use crate::chat::context::Context;
use crate::chat::context::StreamEndReason;
use crate::chat::converter::*;
use crate::openrouter::{self, FinishReason};

#[allow(unused_variables)]
pub trait ChatInner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String>;
    fn get_model(ctx: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model>;
    /// return true if the tool call finalize this completion
    fn handoff_tool<'a>(
        pipeline: &'a mut ChatPipeline<Self>,
        toolcall: Vec<openrouter::ToolCall>,
    ) -> BoxFuture<'a, Result<bool, anyhow::Error>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            Err(anyhow::anyhow!(
                "Tool calls are not supported in this pipeline"
            ))
        })
    }
    fn get_tools(
        ctx: &Context,
        completion_ctx: &CompletionContext,
    ) -> Result<Vec<openrouter::Tool>> {
        Ok(vec![])
    }
}

pub struct ChatPipeline<P: ChatInner> {
    pub ctx: Arc<Context>,
    pub completion_ctx: CompletionContext,
    pub model: openrouter::Model,
    // TODO: don't store message, compute when used
    pub messages: Vec<openrouter::Message>,
    tools: Vec<openrouter::Tool>,
    pipeline: PhantomData<P>,
}

impl<P: ChatInner> ChatPipeline<P> {
    async fn new(ctx: Arc<Context>, completion_ctx: CompletionContext) -> Result<Self> {
        let system_prompt = P::get_system_prompt(&ctx, &completion_ctx)?;

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        for m in &completion_ctx.messages {
            messages.extend(db_message_to_openrouter(&ctx, &m.inner).await?);
        }

        let model = P::get_model(&ctx, &completion_ctx)?;
        let tools = P::get_tools(&ctx, &completion_ctx)?;

        Ok(Self {
            ctx,
            completion_ctx,
            messages,
            model,
            tools,
            pipeline: PhantomData,
        })
    }
    async fn process(&mut self) -> Result<(), anyhow::Error> {
        let message = self.messages.clone();

        let mut res = self
            .ctx
            .openrouter
            .stream(message, &self.model, self.tools.clone())
            .await?;

        let halt_with_error = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(openrouter_to_buffer_token)))
            .await;

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        self.completion_ctx
            .message
            .inner
            .as_assistant()
            .unwrap()
            .extend(openrouter_stream_to_assitant_chunk(&result.responses));

        let annotations = result
            .annotations
            .map(|v| serde_json::to_string(&v).ok())
            .flatten();

        let halt = halt_with_error?;
        if matches!(halt, StreamEndReason::Halt) {
            log::debug!("The stream was halted");
        }

        let mut finalized = true;
        match result.stop_reason {
            FinishReason::Stop => {}
            FinishReason::Error => log::warn!("cannot capture error"),
            FinishReason::Length => log::warn!("The response is too long"),
            FinishReason::ToolCalls => {
                if result.toolcalls.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No tool calls found, but finish reason is tool_calls"
                    ));
                }

                // handoff_tool should decide whether to insert tool_call or not
                finalized = P::handoff_tool(self, result.toolcalls).await?;
                if let Some(annotations) = annotations {
                    self.completion_ctx
                        .message
                        .inner
                        .add_annotation(annotations);
                }
            }
        };

        if finalized {
            return Ok(());
        }
        Box::pin(self.process()).await
    }
}

impl<P: ChatInner + Send> super::Pipeline for ChatPipeline<P> {
    fn process(
        ctx: Arc<Context>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, anyhow::Result<()>> {
        Box::pin(async move {
            let mut pipeline = ChatPipeline::<P>::new(ctx, completion_ctx)
                .await
                .context("Failed to create chat pipeline")?;

            if let Some(err) = pipeline.process().await.err() {
                pipeline.completion_ctx.add_error(err.to_string());
            }
            pipeline.completion_ctx.save().await?;

            Ok(())
        })
    }
}
