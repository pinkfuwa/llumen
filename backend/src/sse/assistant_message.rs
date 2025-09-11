use crate::sse::{EndKind, Publisher};

use anyhow::Result;
use entity::{ChunkKind, MessageKind, ToolCall, chunk, message, prelude::*};
use sea_orm::{ActiveValue::Set, EntityTrait};

use super::Token;

pub struct AssistantMessage<'a> {
    message_id: i32,
    ctx: &'a Publisher,
}

impl<'a> AssistantMessage<'a> {
    pub fn new(message_id: i32, ctx: &'a Publisher) -> Self {
        Self { message_id, ctx }
    }

    pub async fn new_buffer_chunk<'b: 'c, 'c>(&'b self, kind: ChunkKind) -> BufferChunk<'c, 'b> {
        let mut inner = self.ctx.inner.write().await;

        inner.buffer.clear();
        inner.is_reasoning = kind == ChunkKind::Reasoning;

        BufferChunk { ctx: self, kind }
    }

    pub async fn end_message(self, kind: EndKind) -> Result<()> {
        Message::update(message::ActiveModel {
            id: Set(self.message_id),
            kind: Set(MessageKind::Assistant),
            ..Default::default()
        })
        .exec(&self.ctx.conn)
        .await?;

        let mut inner = self.ctx.inner.write().await;
        inner.last_message_id = self.message_id + 1;
        self.ctx
            .raw_token(Ok(Token::MessageEnd(self.message_id, kind)));
        inner.on_receive.notify_waiters();
        Ok(())
    }

    pub fn start_tool_call(&self, name: &'static str, args: String) {
        self.ctx.raw_token(Ok(Token::ToolCall(name, args)));
    }

    pub async fn end_tool_call(
        &self,
        name: &'static str,
        args: String,
        content: String,
        id: String,
    ) -> Result<i32> {
        let chunk_content = serde_json::to_string(&ToolCall {
            id,
            name: name.to_owned(),
            args: args.clone(),
            content: content.clone(),
        })?;
        let id = Chunk::insert(chunk::ActiveModel {
            content: Set(chunk_content),
            kind: Set(ChunkKind::ToolCall),
            message_id: Set(self.message_id),
            ..Default::default()
        })
        .exec(&self.ctx.conn)
        .await?
        .last_insert_id;
        self.ctx
            .raw_token(Ok(Token::ToolCallEnd(name, args, content, id)));

        Ok(id)
    }
}

pub struct BufferChunk<'a, 'b: 'a> {
    ctx: &'a AssistantMessage<'b>,
    kind: ChunkKind,
}

impl<'a, 'b: 'a> BufferChunk<'a, 'b> {
    pub async fn end_buffer_chunk(self, end_kind: EndKind) -> Result<()> {
        let inner = self.ctx.ctx.inner.write().await;
        let context = inner.buffer.clone();
        let id = Chunk::insert(chunk::ActiveModel {
            content: Set(context),
            kind: Set(self.kind),
            message_id: Set(self.ctx.message_id),
            ..Default::default()
        })
        .exec(&self.ctx.ctx.conn)
        .await?
        .last_insert_id;

        self.ctx.ctx.raw_token(Ok(Token::ChunkEnd(id, end_kind)));
        inner.on_receive.notify_waiters();
        Ok(())
    }

    pub async fn send_token(&self, token: &str) -> Result<()> {
        let mut inner = self.ctx.ctx.inner.write().await;

        inner.buffer.push_str(token);
        inner.on_receive.notify_waiters();
        Ok(())
    }

    pub fn kind(&self) -> ChunkKind {
        self.kind
    }
}
