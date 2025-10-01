use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::{Context as _, Result};

use entity::ChunkKind;
use entity::FileHandle;
use entity::MessageKind;
use entity::chunk;
use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use sea_orm::ActiveValue;
use sea_orm::ActiveValue::Set;

use crate::chat::context::CompletionContext;
use crate::chat::context::Context;
use crate::chat::context::StreamEndReason;
use crate::chat::token::Token;
use crate::chat::token::ToolCallInfo;
use crate::openrouter::{self, FinishReason};

use super::helper;

pub trait ChatInner {
    fn get_system_prompt(ctx: &Context, completion_ctx: &CompletionContext) -> Result<String>;
    fn get_model(ctx: &Context, completion_ctx: &CompletionContext) -> Result<openrouter::Model>;
    fn solve_tool(name: &str, arg: &str) -> BoxFuture<'static, Result<String, anyhow::Error>> {
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
    ctx: Arc<Context>,
    completion_ctx: CompletionContext,
    messages: Vec<openrouter::Message>,
    model: openrouter::Model,
    tools: Vec<openrouter::Tool>,
    pipeline: PhantomData<P>,
}

impl<P: ChatInner> ChatPipeline<P> {
    async fn new(ctx: Arc<Context>, completion_ctx: CompletionContext) -> Result<Self> {
        let system_prompt = P::get_system_prompt(&ctx, &completion_ctx)?;

        let mut messages = vec![openrouter::Message::System(system_prompt)];

        for (message, chunks) in &completion_ctx.messages_with_chunks {
            match message.kind {
                MessageKind::Hidden => continue,
                MessageKind::User => {
                    let files = chunks
                        .iter()
                        .filter(|x| matches!(x.kind, ChunkKind::File))
                        .filter_map(|x| serde_json::from_str::<FileHandle>(&x.content).ok())
                        .collect::<Vec<_>>();
                    let files = helper::load_files(ctx.blob.clone(), &files).await?;

                    let text = chunks
                        .iter()
                        .filter(|x| matches!(x.kind, ChunkKind::Text))
                        .map(|x| x.content.as_str())
                        .collect::<String>();
                    match files.is_empty() {
                        true => messages.push(openrouter::Message::User(text)),
                        false => messages.push(openrouter::Message::MultipartUser { text, files }),
                    };
                }
                MessageKind::Assistant => {
                    messages.extend(helper::chunks_to_message(chunks.clone().into_iter()))
                }
                MessageKind::DeepResearch => todo!("Handle DeepResearch message kind"),
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
    fn update_tool_result(&mut self, output: String) -> Result<()> {
        let last_chunk = self
            .completion_ctx
            .new_chunks
            .last_mut()
            .context("No tool chunks found")?;

        debug_assert!(matches!(last_chunk.kind.as_ref(), ChunkKind::ToolCall));

        let content = last_chunk.content.take().unwrap();

        let mut info: ToolCallInfo =
            serde_json::from_str(&content).context("Failed to parse tool call info")?;

        info.output = Some(output);

        last_chunk.content = ActiveValue::set(
            serde_json::to_string(&info).context("Failed to serialize tool call info")?,
        );

        Ok(())
    }
    async fn process(&mut self) -> Result<(), anyhow::Error> {
        let mut message = self.messages.clone();
        message.extend(helper::active_chunks_to_message(
            self.completion_ctx.new_chunks.clone().into_iter(),
        ));

        let mut res = self
            .ctx
            .openrouter
            .stream(message, &self.model, self.tools.clone())
            .await?;

        let halt = self
            .completion_ctx
            .put_stream((&mut res).map(|resp| resp.map(Into::into)))
            .await?;

        if matches!(halt, StreamEndReason::Halt) {
            return Err(anyhow::anyhow!("The stream was halted"));
        }

        let result = res.get_result();

        self.completion_ctx
            .update_usage(result.usage.cost as f32, result.usage.token as i32);

        let tokens = result.responses.into_iter().map(Into::into);
        let mut chunks = Token::into_chunks(tokens.into_iter()).collect::<Vec<_>>();

        if let Some(annotations) = result
            .annotations
            .map(|v| serde_json::to_string(&v).ok())
            .flatten()
        {
            if let Some(idx) = chunks
                .iter()
                .rposition(|elem| matches!(elem.kind, ActiveValue::Set(ChunkKind::Text)))
            {
                chunks.insert(
                    idx + 1,
                    chunk::ActiveModel {
                        content: Set(annotations),
                        kind: Set(ChunkKind::Annotation),
                        ..Default::default()
                    },
                );
            }
        }

        self.completion_ctx.new_chunks.extend(chunks);

        match result.stop_reason {
            FinishReason::Length => return Err(anyhow::anyhow!("The response is too long")),
            FinishReason::ToolCalls => {
                let tool_call = result
                    .toolcall
                    .context("No tool calls found, but finish reason is tool_calls")?;

                let result = P::solve_tool(&tool_call.name, &tool_call.args).await?;

                self.update_tool_result(result)?;

                return Box::pin(self.process()).await;
            }
            FinishReason::Stop => return Ok(()),
        };
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

            let err = pipeline.process().await.err();
            pipeline.completion_ctx.save(err).await?;

            Ok(())
        })
    }
}
