use std::{collections::hash_map::Entry, sync::Arc};

use anyhow::Result;
use entity::{message, patch::MessageKind, prelude::*};
use sea_orm::{ActiveValue::Set, DbConn, EntityTrait};
use tokio::sync::{Notify, RwLock, broadcast};

use crate::{
    errors::*,
    sse::{SseContext, SseInner, Token},
};

#[derive(Debug)]
pub struct Publisher {
    chat_id: i32,
    channel: broadcast::Sender<Result<Token, Error>>,
    inner: Arc<RwLock<SseInner>>,
    on_halt: Arc<Notify>,
    conn: DbConn,
}

#[derive(Debug, Clone, Copy)]
pub enum PublisherKind {
    Assistant,
    Reasoning,
}

impl Publisher {
    pub async fn on_halt(&self) {
        self.on_halt.notified().await;
    }

    pub async fn token(&mut self, t: &str) {
        let mut inner = self.inner.write().await;
        inner.buffer.push_str(t);

        inner.on_receive.notify_waiters();
    }

    pub async fn user_message(&mut self, t: String) -> Result<i32> {
        let res = Message::insert(message::ActiveModel {
            chat_id: Set(self.chat_id),
            text: Set(Some(t.clone())),
            kind: Set(MessageKind::User),
            ..Default::default()
        })
        .exec(&self.conn)
        .await?;

        self.inner.write().await.last_id = res.last_insert_id;
        self.raw_token(Ok(Token::User(res.last_insert_id, t)));
        Ok(res.last_insert_id)
    }
    pub fn raw_token(&self, t: Result<Token, Error>) {
        self.channel.send(t).ok();
    }

    pub async fn close(self) {
        let mut inner = self.inner.write().await;
        let Some(id) = inner.db_id else {
            drop(inner);
            self.raw_token(Err(Error {
                error: ErrorKind::Internal,
                reason: "Publisher is in a undefine state".to_owned(),
            }));

            return;
        };
        let text = inner.buffer.clone();
        let res = Message::update(message::ActiveModel {
            id: Set(id),
            text: Set(Some(text)),
            ..Default::default()
        })
        .exec(&self.conn)
        .await;

        if let Err(e) = res {
            drop(inner);
            self.raw_token(Err(Error {
                error: ErrorKind::Internal,
                reason: e.to_string(),
            }));
            return;
        }
        inner.last_id += 1;
        self.raw_token(Ok(Token::End(id)));

        inner.on_receive.notify_waiters();
    }

    pub async fn new_stream(&self, kind: PublisherKind) {
        let mut inner = self.inner.write().await;

        let db_kind = match kind {
            PublisherKind::Assistant => MessageKind::Assistant,
            PublisherKind::Reasoning => MessageKind::Reasoning,
        };

        let res = Message::insert(message::ActiveModel {
            chat_id: Set(self.chat_id),
            kind: Set(db_kind),
            ..Default::default()
        })
        .exec(&self.conn)
        .await;

        let db_id = match res {
            Ok(v) => v.last_insert_id,
            Err(e) => {
                self.raw_token(Err(Error {
                    error: ErrorKind::Internal,
                    reason: e.to_string(),
                }));
                return;
            }
        };

        inner.buffer.clear();
        inner.db_id = Some(db_id);
        inner.last_id = db_id;
    }

    pub(super) async fn new(ctx: &SseContext, chat_id: i32) -> Result<Self> {
        match ctx.map.lock().await.entry(chat_id) {
            Entry::Occupied(entry) => {
                let inner = entry.get().write().await;

                let channel = inner.channel.clone();
                let on_halt = inner.on_halt.clone();
                let inner = entry.get().clone();

                Ok(Self {
                    channel,
                    inner,
                    on_halt,
                    conn: ctx.conn.clone(),
                    chat_id,
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
                })
            }
        }
    }
}
