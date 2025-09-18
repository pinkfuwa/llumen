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
use futures_util::{Stream, StreamExt, stream};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, chat::Token, errors::*, middlewares::auth::UserId};

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
    pub version: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToken {
    pub content: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespChunkEnd {
    pub kind: SseRespEndKind,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespMessageEnd {
    pub id: i32,
    pub chunk_ids: Vec<i32>,
    pub token_count: i32,
    pub cost: f32,
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

    let stream = pipeline.clone().subscribe(req.id);

    let last_message = Message::find()
        .filter(entity::message::Column::ChatId.eq(req.id))
        .order_by_desc(entity::message::Column::Id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    let initial_event = if let Some(last_message) = last_message {
        let event = SseResp::LastMessage(SseRespLastMessage {
            id: last_message.id,
            // FIXME: change version when revalidate is needed
            version: last_message.id,
        });
        let event = Event::default().json_data(event).unwrap();
        Some(Ok(event))
    } else {
        None
    };

    let st = stream::iter(initial_event).chain(stream.map(|token| {
        let event = match token {
            Token::Assitant(content) => SseResp::Token(SseRespToken { content }),
            Token::Reasoning(content) => SseResp::ReasoningToken(SseRespToken { content }),
            Token::Tool { name, args, .. } => SseResp::ToolCall(SseRespToolCall { name, args }),
            Token::Complete {
                message_id,
                chunk_ids,
                token,
                cost,
            } => SseResp::MessageEnd(SseRespMessageEnd {
                id: message_id,
                chunk_ids,
                token_count: token,
                cost,
            }),
            Token::ToolResult(content) => SseResp::ToolCallEnd(SseRespToolCallEnd { content }),
            _ => return Ok(Event::default()),
        };
        Ok(Event::default().json_data(event).unwrap())
    }));

    Ok(Sse::new(st).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    ))
}
