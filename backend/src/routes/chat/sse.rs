use std::{sync::Arc, time::Duration};

use crate::chat::Token;
use axum::{
    Extension, Json,
    extract::State,
    response::{
        Sse,
        sse::{Event, KeepAlive},
    },
};
use entity::prelude::*;
use futures_util::{Stream, StreamExt};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum SseResp {
    LastMessage(SseRespLastMessage),

    Token(SseRespToken),
    ReasoningToken(SseRespToken),
    ChunkEnd(SseRespChunkEnd),

    ToolCall(SseRespToolCall),
    ToolCallEnd(SseRespToolCallEnd),

    MessageEnd(SseRespMessageEnd),

    UserMessage(SseRespUserMessage),

    ChangeTitle(SseRespUserTitle),

    Usage(SseRespUsage),
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespUsage {
    token: u32,
    price: f32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespUserTitle {
    pub title: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespLastMessage {
    pub id: i32,
    pub version: u32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToken {
    pub content: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespChunkEnd {
    pub id: i32,
    pub kind: SseRespEndKind,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespMessageEnd {
    pub id: i32,
    pub kind: SseRespEndKind,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum SseRespEndKind {
    Complete,
    Halt,
    Error,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespUserMessage {
    pub message_id: i32,
    pub chunk_id: i32,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToolCall {
    pub name: String,
    pub args: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToolCallEnd {
    pub chunk_id: i32,
    pub name: String,
    pub args: String,
    pub content: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<SseReq>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, Json<Error>> {
    let pipeline = app.pipeline.clone();
    let res = Chat::find_by_id(req.id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or("")
        .kind(ErrorKind::ResourceNotFound)?;

    if res.owner_id != user_id {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

    let st = pipeline
        .subscribe(req.id)
        .map(|x| SseResp::from(x))
        .map(|x| todo!("We need a stateful mapper"));

    Ok(Sse::new(st).keep_alive(KeepAlive::new().interval(Duration::from_secs(10))))
}

impl From<Token> for SseResp {
    fn from(value: Token) -> Self {
        todo!()
    }
}
