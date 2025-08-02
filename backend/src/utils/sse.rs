use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use anyhow::{Context, Result, anyhow};
use entity::{message, prelude::*};
use sea_orm::{ActiveValue::Set, DbConn, EntityTrait, QueryOrder};
use tokio::{
    spawn,
    sync::{Notify, broadcast, mpsc, oneshot},
};

const MAX_CAP: usize = 100;

#[derive(Debug)]
pub enum Event {
    Subscribe(i32, oneshot::Sender<Subscribe>),
    Publish(i32, oneshot::Sender<Option<Publish>>),
    Token(PublishId, PublishToken),
    Halt(i32),
}

#[derive(Debug)]
pub struct Subscribe {
    pub message: SubscribeMessage,
    pub channel: broadcast::Receiver<SubscribeToken>,
}

#[derive(Debug, Clone)]
pub enum SubscribeMessage {
    Cache(SubscribeMessageCache),
    Last(i32),
}

#[derive(Debug, Clone)]
pub struct SubscribeMessageCache {
    pub message_id: i32,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum SubscribeToken {
    Start(i32),
    Token(String),
    End,
}

#[derive(Debug, Clone)]
pub struct Publish {
    pub id: PublishId,
    pub halt: Arc<Notify>,
    pub channel: mpsc::UnboundedSender<Event>,
}

#[derive(Debug, Clone)]
pub struct PublishId(i32, i32);

#[derive(Debug, Clone)]
pub enum PublishToken {
    End,
    Text(String),
}

pub struct SseContext(mpsc::UnboundedSender<Event>);

#[derive(Debug)]
struct ChannelMap {
    channel: broadcast::Sender<SubscribeToken>,
    message: Option<(i32, String)>,
    halt: Arc<Notify>,
}

impl SseContext {
    pub async fn subscribe(&self, chat_id: i32) -> Result<Subscribe> {
        let (sen, rev) = oneshot::channel();
        self.0.send(Event::Subscribe(chat_id, sen))?;
        let res = rev.await?;

        Ok(res)
    }

    pub async fn publish(&self, chat_id: i32) -> Result<Publish> {
        let (sen, rev) = oneshot::channel();
        self.0.send(Event::Publish(chat_id, sen))?;
        let res = rev
            .await?
            .ok_or(anyhow!("Completion already in progress"))?;

        Ok(res)
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

async fn on_event(
    event: Event,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
    sen: &mpsc::UnboundedSender<Event>,
) -> Result<()> {
    match event {
        Event::Subscribe(chat_id, ret) => on_subscribe(chat_id, ret, map, conn).await,
        Event::Publish(chat_id, ret) => on_publish(chat_id, ret, map, conn, sen).await,
        Event::Token(id, token) => on_token(id, token, map, conn).await,
        Event::Halt(chat_id) => on_halt(chat_id, map).await,
    }
}

async fn on_subscribe(
    chat_id: i32,
    ret: oneshot::Sender<Subscribe>,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
) -> Result<()> {
    match map.entry(chat_id) {
        Entry::Occupied(entry) => {
            let message = if let Some((message_id, message)) = entry.get().message.clone() {
                SubscribeMessage::Cache(SubscribeMessageCache {
                    message_id,
                    message,
                })
            } else {
                let id = Message::find()
                    .order_by_desc(message::Column::Id)
                    .one(conn)
                    .await?
                    .map(|x| x.id + 1)
                    .unwrap_or(0);

                SubscribeMessage::Last(id)
            };
            let channel = entry.get().channel.subscribe();

            ret.send(Subscribe { message, channel }).ok();
        }
        Entry::Vacant(entry) => {
            let (sen, rev) = broadcast::channel(MAX_CAP);
            entry.insert(ChannelMap {
                channel: sen,
                message: None,
                halt: Arc::new(Notify::new()),
            });

            let id = Message::find()
                .order_by_desc(message::Column::Id)
                .one(conn)
                .await?
                .map(|x| x.id + 1)
                .unwrap_or(0);

            ret.send(Subscribe {
                message: SubscribeMessage::Last(id),
                channel: rev,
            })
            .ok();
        }
    }

    Ok(())
}

async fn on_publish(
    chat_id: i32,
    ret: oneshot::Sender<Option<Publish>>,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
    sen: &mpsc::UnboundedSender<Event>,
) -> Result<()> {
    match map.entry(chat_id) {
        Entry::Occupied(mut entry) => {
            if entry.get().message.is_some() {
                // TODO:
                // Add warning

                ret.send(None).ok();
                return Ok(());
            }

            let res = Message::insert(message::ActiveModel {
                chat_id: Set(chat_id),
                text: Set("".to_owned()),
                ..Default::default()
            })
            .exec(conn)
            .await
            .context("Cannot create message")?;

            entry.get_mut().message = Some((res.last_insert_id, "".to_owned()));
            ret.send(Some(Publish {
                id: PublishId(chat_id, res.last_insert_id),
                halt: entry.get().halt.clone(),
                channel: sen.clone(),
            }))
            .ok();
        }
        Entry::Vacant(entry) => {
            let res = Message::insert(message::ActiveModel {
                chat_id: Set(chat_id),
                text: Set("".to_owned()),
                ..Default::default()
            })
            .exec(conn)
            .await
            .context("Cannot create message")?;

            let entry = entry.insert(ChannelMap {
                channel: broadcast::channel(MAX_CAP).0,
                message: Some((res.last_insert_id, "".to_owned())),
                halt: Arc::new(Notify::new()),
            });

            ret.send(Some(Publish {
                id: PublishId(chat_id, res.last_insert_id),
                halt: entry.halt.clone(),
                channel: sen.clone(),
            }))
            .ok();
        }
    }

    Ok(())
}

async fn on_token(
    PublishId(chat_id, msg_id): PublishId,
    token: PublishToken,
    map: &mut HashMap<i32, ChannelMap>,
    conn: &DbConn,
) -> Result<()> {
    let Some(entry) = map.get_mut(&chat_id) else {
        // TODO:
        // add warning
        return Ok(());
    };

    if entry.message.as_ref().is_none_or(|(id, _)| *id != msg_id) {
        // TODO:
        // add warning
        return Ok(());
    }

    match token {
        PublishToken::End => {
            let msg = entry.message.take().unwrap().1;
            Message::update(message::ActiveModel {
                id: Set(msg_id),
                text: Set(msg),
                ..Default::default()
            })
            .exec(conn)
            .await
            .context("Cannot set message")?;
        }
        PublishToken::Text(text) => {
            let Some((_, msg)) = &mut entry.message else {
                unreachable!()
            };

            msg.push_str(&text);
            entry.channel.send(SubscribeToken::Token(text)).ok();
        }
    }
    Ok(())
}

async fn on_halt(chat_id: i32, map: &mut HashMap<i32, ChannelMap>) -> Result<()> {
    let Some(entry) = map.get(&chat_id) else {
        // TODO
        // add warning
        return Ok(());
    };

    entry.halt.notify_waiters();

    Ok(())
}
