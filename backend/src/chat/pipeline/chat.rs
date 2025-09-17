use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::{Context as _, Result};

use entity::ChunkKind;
use entity::MessageKind;
use futures_util::future::BoxFuture;

use crate::chat::context::CompletionContext;
use crate::chat::context::PipelineContext;
use crate::chat::token::Token;
use crate::openrouter::{self, FinishReason};

use super::helper;

pub trait PipelineInner {
    fn get_system_prompt(
        ctx: &PipelineContext,
        completion_ctx: &CompletionContext,
    ) -> Result<String>;
    fn get_model(
        ctx: &PipelineContext,
        completion_ctx: &CompletionContext,
    ) -> Result<openrouter::Model>;
    fn solve_tool(
        tool_call: openrouter::ToolCall,
    ) -> BoxFuture<'static, Result<String, anyhow::Error>> {
        Box::pin(async move {
            Err(anyhow::anyhow!(
                "Tool calls are not supported in this pipeline"
            ))
        })
    }
    fn get_tools(
        ctx: &PipelineContext,
        completion_ctx: &CompletionContext,
    ) -> Result<Vec<openrouter::Tool>> {
        Ok(vec![])
    }
}

pub struct ChatPipeline<P: PipelineInner> {
    ctx: Arc<PipelineContext>,
    completion_ctx: CompletionContext,
    messages: Vec<openrouter::Message>,
    model: openrouter::Model,
    tools: Vec<openrouter::Tool>,
    pipeline: PhantomData<P>,
}

impl<P: PipelineInner> ChatPipeline<P> {
    async fn new(ctx: Arc<PipelineContext>, completion_ctx: CompletionContext) -> Result<Self> {
        let system_prompt = P::get_system_prompt(&ctx, &completion_ctx)?;

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        for (message, chunks) in &completion_ctx.messages_with_chunks {
            match message.kind {
                MessageKind::Hidden => continue,
                MessageKind::User => {
                    let text = chunks
                        .iter()
                        .filter(|x| matches!(x.kind, ChunkKind::Text))
                        .map(|x| x.content.as_str())
                        .collect::<String>();
                    messages.push(openrouter::Message::User(text));
                }
                MessageKind::Assistant => {
                    messages.extend(helper::chunks_to_message(chunks.clone().into_iter()))
                }
                MessageKind::DeepResearch => todo!(),
            };
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
    async fn report_error(&mut self, err: impl ToString) {
        // TODO: report_error can have fail due to publisher halted, we chould at least log it.
        self.completion_ctx
            .add_token(Token::Error(err.to_string()))
            .await;
    }
    async fn process_one(&mut self) -> Result<(), anyhow::Error> {
        loop {
            let mut message = self.messages.clone();
            message.extend(helper::active_chunks_to_message(
                self.completion_ctx.new_chunks.clone().into_iter(),
            ));

            let mut res = self
                .ctx
                .openrouter
                .stream(message, &self.model, self.tools.clone())
                .await?;

            let token_stream = helper::to_token_stream(&mut res);

            let halt = self.completion_ctx.put_stream(token_stream).await?;
            tracing::debug!("stream ended: {:?}", halt);

            match res.stop_reason.clone().context("stream didn't stop")? {
                FinishReason::Length => return Err(anyhow::anyhow!("The response is too long")),
                FinishReason::ToolCalls => {
                    let tool_call = res
                        .toolcall
                        .clone()
                        .context("No tool calls found, but finish reason is tool_calls")?;
                    P::solve_tool(tool_call).await?;
                }
                _ => {}
            };
        }
    }
}

impl<P: PipelineInner + Send> super::Pipeline for ChatPipeline<P> {
    fn process(
        ctx: Arc<PipelineContext>,
        completion_ctx: CompletionContext,
    ) -> BoxFuture<'static, anyhow::Result<()>> {
        Box::pin(async move {
            let mut pipeline = ChatPipeline::<P>::new(ctx, completion_ctx)
                .await
                .context("Failed to create chat pipeline")?;

            if let Err(err) = pipeline.process_one().await {
                pipeline.report_error(err).await;
            }

            Ok(())
        })
    }
}
