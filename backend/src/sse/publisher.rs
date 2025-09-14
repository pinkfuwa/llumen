use std::{
    collections::hash_map::Entry,
    sync::{Arc, Mutex},
};

use anyhow::{Result, bail};
use entity::{MessageKind, chunk, message, patch::ChunkKind, prelude::*};
use futures_util::FutureExt;
use sea_orm::{ActiveValue::Set, TransactionTrait, prelude::*};
use tokio::sync::{Notify, RwLock, broadcast};

use crate::{
    config,
    errors::*,
    sse::{AssistantMessage, SseContext, SseInner, Token},
};

use super::buffer::Buffer;

#[derive(Debug)]
pub struct Publisher {
    pub(super) chat_id: i32,
    channel: broadcast::Sender<Result<Token, Error>>,
    pub(super) inner: Arc<RwLock<SseInner>>,
    pub(super) on_halt: Arc<Notify>,
    pub(super) conn: DbConn,
    pub(super) buffer: Mutex<Buffer<Result<Token, Error>>>,
}

impl Publisher {
    pub async fn scope<'a, T, F>(&'a self, func: impl FnOnce(&'a Self) -> F) -> Option<T>
    where
        F: Future<Output = Result<T, Error>>,
    {
        let res = func(self).await;
        match res {
            Ok(v) => return Some(v),
            Err(err) => self.raw_token(Err(err)),
        }
        None
    }

    pub async fn on_halt(&self) {
        self.on_halt.notified().await;
    }

    pub async fn user_message(&self, t: String) -> Result<i32> {
        let (message_id, chunk_id) = self
            .conn
            .transaction(|conn| {
                let chat_id = self.chat_id;
                let t = t.clone();
                async move {
                    let message_id = Message::insert(message::ActiveModel {
                        chat_id: Set(chat_id),
                        kind: Set(MessageKind::User),
                        ..Default::default()
                    })
                    .exec(conn)
                    .await?
                    .last_insert_id;

                    let chunk_id = Chunk::insert(chunk::ActiveModel {
                        content: Set(t),
                        kind: Set(ChunkKind::Text),
                        message_id: Set(message_id),
                        ..Default::default()
                    })
                    .exec(conn)
                    .await?
                    .last_insert_id;

                    Result::<_>::Ok((message_id, chunk_id))
                }
                .boxed()
            })
            .await?;

        self.inner.write().await.last_message_id = message_id + 1;
        self.raw_token(Ok(Token::UserMessage(message_id, chunk_id, t)));
        Ok(message_id)
    }

    pub fn error(&self, e: Error) {
        self.raw_token(Err(e));
    }

    pub fn raw_token(&self, t: Result<Token, Error>) {
        let mut new_token = Some(t);
        let mut chan_len = self.channel.len();
        let mut buffer = self.buffer.lock().unwrap();
        while chan_len < config::MAX_SSE_BUF {
            if let Some(token) = buffer.pop() {
                chan_len += 1;
                self.channel.send(token).ok();
            } else if let Some(token) = new_token.take() {
                self.channel.send(token).ok();
            } else {
                break;
            }
        }
        if let Some(token) = new_token {
            buffer.push(token);
        }
    }

    pub async fn new_assistant_message<'a>(&'a self) -> Result<AssistantMessage<'a>> {
        let message_id = Message::insert(message::ActiveModel {
            chat_id: Set(self.chat_id),
            kind: Set(MessageKind::Assistant),
            ..Default::default()
        })
        .exec(&self.conn)
        .await?
        .last_insert_id;

        Ok(AssistantMessage::new(message_id, self))
    }

    pub(super) async fn new(ctx: &SseContext, chat_id: i32) -> Result<Self> {
        match ctx.map.lock().await.entry(chat_id) {
            Entry::Occupied(entry) => {
                let inner = entry.get().write().await;
                if inner.channel.strong_count() != 1 {
                    bail!("Only 1 publisher can exisit at the same time");
                }

                let channel = inner.channel.clone();
                let on_halt = inner.on_halt.clone();
                let inner = entry.get().clone();

                Ok(Self {
                    channel,
                    inner,
                    on_halt,
                    conn: ctx.conn.clone(),
                    chat_id,
                    buffer: Mutex::new(Buffer::default()),
                })
            }
            Entry::Vacant(entry) => {
                let inner = SseInner::new(ctx).await?;
                let channel = inner.channel.clone();
                let on_halt = inner.on_halt.clone();
                let inner = entry.insert(Arc::new(RwLock::new(inner))).clone();

                Ok(Self {
                    channel,
                    inner,
                    on_halt,
                    conn: ctx.conn.clone(),
                    chat_id,
                    buffer: Mutex::new(Buffer::default()),
                })
            }
        }
    }
}

impl Drop for Publisher {
    fn drop(&mut self) {
        let mut buffer = self.buffer.lock().unwrap();
        while let Some(token) = buffer.pop() {
            self.channel.send(token).ok();
        }
    }
}
