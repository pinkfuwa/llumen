use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};
use entity::{message, patch::MessageKind, prelude::*};
use sea_orm::{ActiveValue::Set, DbConn, EntityTrait, QueryOrder};
use tokio::{
    spawn,
    sync::{Notify, broadcast, mpsc, oneshot},
};
use tracing::{instrument, warn};

use crate::{config, errors::Error};

#[derive(Debug)]
pub enum Event {
    Subscribe(i32, oneshot::Sender<Result<Subscribe>>),
    Publish(i32, oneshot::Sender<Result<Option<Publish>>>),
    Token(i32, PublishToken),
    Halt(i32),
}

#[derive(Debug)]
pub struct Subscribe {
    pub last: i32,
    pub message: Option<String>,
    pub channel: broadcast::Receiver<SubscribeToken>,
}

#[derive(Debug, Clone)]
pub enum SubscribeToken {
    Token(String),
    UserText(i32, String),
    Error(Error),
    End(i32),
}

#[derive(Debug, Clone)]
pub struct Publish {
    pub halt: Arc<Notify>,
    pub channel: PublishChannel,
}

#[derive(Debug, Clone)]
pub struct PublishChannel(i32, mpsc::UnboundedSender<Event>);

#[derive(Debug, Clone)]
pub enum PublishToken {
    End,
    Text(String),
    UserText(i32, String),
    Error(Error),
}

pub struct SseContext(mpsc::UnboundedSender<Event>);

#[derive(Debug)]
struct ChannelMap {
    channel: broadcast::Sender<SubscribeToken>,
    last: i32,
    message: Option<String>,
    halt: Arc<Notify>,
}

impl SseContext {
    pub async fn subscribe(&self, chat_id: i32) -> Result<Subscribe> {
        let (sen, rev) = oneshot::channel();
        self.0.send(Event::Subscribe(chat_id, sen))?;
        let res = rev.await??;

        Ok(res)
    }

    pub async fn publish(&self, chat_id: i32) -> Result<Publish> {
        let (sen, rev) = oneshot::channel();
        self.0.send(Event::Publish(chat_id, sen))?;
        let res = rev
            .await??
            .ok_or_else(|| anyhow!("Completion already in progress"))?;

        Ok(res)
    }

    pub fn halt(&self, chat_id: i32) -> Result<()> {
        self.0.send(Event::Halt(chat_id))?;
        Ok(())
    }
}

impl PublishChannel {
    pub fn send(&self, token: PublishToken) {
        self.1.send(Event::Token(self.0, token)).ok();
    }
}

pub fn spawn_sse(conn: DbConn) -> SseContext {
    let (sen, mut rev) = mpsc::unbounded_channel();

    spawn({
        let sen = sen.clone();
        async move {
            let mut map = HashMap::new();
            loop {
                let event = rev.recv().await.expect("Sse channel has been closed");
                on_event(event, &mut map, &conn, &sen)
                    .await
                    .expect("Error in sse");
            }
        }
    });

    SseContext(sen)
}

#[instrument]
async fn on_event(
    event: Event,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
    sen: &mpsc::UnboundedSender<Event>,
) -> Result<()> {
    match event {
        Event::Subscribe(chat_id, ret) => {
            ret.send(on_subscribe(chat_id, map, conn).await).ok();
        }
        Event::Publish(chat_id, ret) => {
            ret.send(on_publish(chat_id, map, conn, sen).await).ok();
        }
        Event::Token(id, token) => {
            on_token(id, token, map, conn).await?;
        }
        Event::Halt(chat_id) => {
            on_halt(chat_id, map).await?;
        }
    };
    Ok(())
}

async fn on_subscribe(
    chat_id: i32,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
) -> Result<Subscribe> {
    let ret = match map.entry(chat_id) {
        Entry::Occupied(entry) => {
            let message = entry.get().message.clone();
            let last = entry.get().last;
            let channel = entry.get().channel.subscribe();
            Subscribe {
                message,
                channel,
                last,
            }
        }
        Entry::Vacant(entry) => {
            let last = Message::find()
                .order_by_desc(message::Column::Id)
                .one(conn)
                .await?
                .map(|x| x.id + 1)
                .unwrap_or(0);

            let (sen, rev) = broadcast::channel(config::MAX_SSE_BUF);
            entry.insert(ChannelMap {
                channel: sen,
                message: None,
                halt: Arc::new(Notify::new()),
                last,
            });
            Subscribe {
                message: None,
                channel: rev,
                last,
            }
        }
    };

    Ok(ret)
}

async fn on_publish(
    chat_id: i32,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
    sen: &mpsc::UnboundedSender<Event>,
) -> Result<Option<Publish>> {
    let ret = match map.entry(chat_id) {
        Entry::Occupied(entry) => {
            if entry.get().message.is_some() {
                return Ok(None);
            }

            Some(Publish {
                halt: entry.get().halt.clone(),
                channel: PublishChannel(chat_id, sen.clone()),
            })
        }
        Entry::Vacant(entry) => {
            let last = Message::find()
                .order_by_desc(message::Column::Id)
                .one(conn)
                .await?
                .map(|x| x.id + 1)
                .unwrap_or(0);

            let entry = entry.insert(ChannelMap {
                channel: broadcast::channel(config::MAX_SSE_BUF).0,
                message: None,
                halt: Arc::new(Notify::new()),
                last,
            });

            Some(Publish {
                halt: entry.halt.clone(),
                channel: PublishChannel(chat_id, sen.clone()),
            })
        }
    };

    Ok(ret)
}

#[instrument]
async fn on_token(
    chat_id: i32,
    token: PublishToken,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
) -> Result<()> {
    let Some(entry) = map.get_mut(&chat_id) else {
        warn!("Cannot find entry: {chat_id}");
        return Ok(());
    };

    match token {
        PublishToken::End => {
            let Some(text) = entry.message.take() else {
                warn!("Cannot end message: {chat_id}");
                return Ok(());
            };

            let res = Message::insert(message::ActiveModel {
                chat_id: Set(chat_id),
                text: Set(text),
                kind: Set(MessageKind::Assistant),
                ..Default::default()
            })
            .exec(conn)
            .await
            .context("Cannot create message")?;

            entry.last = res.last_insert_id;
            entry
                .channel
                .send(SubscribeToken::End(res.last_insert_id))
                .ok();
        }
        PublishToken::Text(text) => {
            let msg = entry.message.get_or_insert_default();

            msg.push_str(&text);
            entry.channel.send(SubscribeToken::Token(text)).ok();
        }
        PublishToken::UserText(id, text) => {
            entry.last = id;
            entry.channel.send(SubscribeToken::UserText(id, text)).ok();
        }
        PublishToken::Error(error) => {
            entry.message = None;
            entry.channel.send(SubscribeToken::Error(error)).ok();
        }
    }
    Ok(())
}

async fn on_halt(chat_id: i32, map: &mut HashMap<i32, ChannelMap>) -> Result<()> {
    let Some(entry) = map.get(&chat_id) else {
        return Ok(());
    };

    entry.halt.notify_waiters();

    Ok(())
}
