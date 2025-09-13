use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use entity::{message, prelude::*};
use sea_orm::{DbConn, EntityTrait, QueryOrder};
use serde::Serialize;
use tokio::sync::{Mutex, Notify, RwLock, broadcast};

use super::subscriber::Subscriber;
use crate::{config::MAX_SSE_BUF, errors::Error, sse::Publisher};

#[derive(Debug, Clone)]
pub struct SseContext {
    pub(super) map: Arc<Mutex<HashMap<i32, Arc<RwLock<SseInner>>>>>,
    pub(super) conn: DbConn,
}

#[derive(Debug, Clone)]
pub struct SseInner {
    /// Last message id
    /// Default to last message id in DB
    /// for pagination
    pub last_message_id: i32,

    /// A random version
    /// When update it will +1
    pub version: u32,

    /// `on_receive` will notify when buffer/id change
    pub on_receive: Arc<Notify>,
    pub is_reasoning: bool,
    pub buffer: String,

    /// Extra token
    pub channel: broadcast::Sender<Result<Token, Error>>,

    /// on halt completion
    pub on_halt: Arc<Notify>,
}

impl SseInner {
    pub async fn new(ctx: &SseContext) -> Result<Self> {
        let last_id = Message::find()
            .order_by_desc(message::Column::Id)
            .one(&ctx.conn)
            .await?
            .map(|x| x.id + 1)
            .unwrap_or(0);
        let version = fastrand::u32(0..u16::MAX as u32);
        Ok(Self {
            buffer: "".to_owned(),
            last_message_id: last_id,
            version,
            on_receive: Arc::new(Notify::new()),
            on_halt: Arc::new(Notify::new()),
            channel: broadcast::channel(MAX_SSE_BUF).0,
            is_reasoning: true,
        })
    }
}

impl SseContext {
    pub fn new(conn: DbConn) -> Self {
        Self {
            map: Default::default(),
            conn,
        }
    }
    pub async fn subscribe(&self, chat_id: i32) -> Result<Subscriber> {
        Subscriber::new(self, chat_id).await
    }

    pub async fn publish(&self, chat_id: i32) -> Result<Publisher> {
        Publisher::new(self, chat_id).await
    }

    pub async fn halt(&self, chat_id: i32) {
        let map = self.map.lock().await;

        let Some(v) = map.get(&chat_id) else {
            return;
        };

        v.read().await.on_halt.notify_waiters();
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Token {
    // id, version
    LastMessage(i32, u32),
    Token(String),
    ReasoningToken(String),

    /// End token
    ChunkEnd(i32, EndKind),
    MessageEnd(i32, EndKind),

    /// message id, chunk id, content
    UserMessage(i32, i32, String),

    /// name, args
    ToolCall(&'static str, String),
    /// name, args, context, id
    ToolCallEnd(&'static str, String, String, i32),

    // change title
    ChangeTitle(String),
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum EndKind {
    Complete,
    Halt,
    Error,
}
