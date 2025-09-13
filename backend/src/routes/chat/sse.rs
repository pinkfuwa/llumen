use std::{sync::Arc, time::Duration};

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

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    sse::{EndKind, Token},
};

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

    let sub = app
        .sse
        .subscribe(req.id)
        .await
        .kind(ErrorKind::MalformedRequest)?;
    let st = sub
        .map(|x| {
            x.map(|v| match v {
                Token::LastMessage(id, version) => {
                    SseResp::LastMessage(SseRespLastMessage { id, version })
                }
                Token::Token(content) => SseResp::Token(SseRespToken { content }),
                Token::ReasoningToken(content) => SseResp::ReasoningToken(SseRespToken { content }),
                Token::ChunkEnd(id, end_kind) => SseResp::ChunkEnd(SseRespChunkEnd {
                    id,
                    kind: match end_kind {
                        EndKind::Complete => SseRespEndKind::Complete,
                        EndKind::Halt => SseRespEndKind::Halt,
                        EndKind::Error => SseRespEndKind::Error,
                    },
                }),
                Token::MessageEnd(id, end_kind) => SseResp::MessageEnd(SseRespMessageEnd {
                    id,
                    kind: match end_kind {
                        EndKind::Complete => SseRespEndKind::Complete,
                        EndKind::Halt => SseRespEndKind::Halt,
                        EndKind::Error => SseRespEndKind::Error,
                    },
                }),
                Token::UserMessage(message_id, chunk_id, content) => {
                    SseResp::UserMessage(SseRespUserMessage {
                        message_id,
                        chunk_id,
                        content,
                    })
                }
                Token::ToolCall(name, args) => SseResp::ToolCall(SseRespToolCall {
                    name: name.to_owned(),
                    args,
                }),
                Token::ToolCallEnd(name, args, content, chunk_id) => {
                    SseResp::ToolCallEnd(SseRespToolCallEnd {
                        chunk_id,
                        name: name.to_owned(),
                        args,
                        content,
                    })
                }
                Token::ChangeTitle(title) => SseResp::ChangeTitle(SseRespUserTitle { title }),
            })
        })
        .map(|x| Event::default().json_data(JsonUnion::from(x)));
    Ok(Sse::new(st).keep_alive(KeepAlive::new().interval(Duration::from_secs(10))))
}
