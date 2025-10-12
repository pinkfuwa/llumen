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
use futures_util::stream;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};
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
    Version(SseRespVersion),
    Token(SseRespToken),
    Reasoning(SseRespReasoning),
    ToolCall(SseRespToolCall),
    ToolResult(SseRespToolResult),
    Complete(SseRespMessageComplete),
    Title(SseRespTitle),
    Error(SseRespError),
}

#[derive(Debug, Serialize)]
#[typeshare]
pub enum LastKind {
    User,
    Assistant,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespVersion {
    pub version: i32,
    pub last_kind: LastKind,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToken {
    pub content: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespReasoning {
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
pub struct SseRespToolResult {
    pub content: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespMessageComplete {
    pub id: i32,
    pub chunk_ids: Vec<i32>,
    pub token_count: i32,
    pub cost: f32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespTitle {
    pub title: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespError {
    pub content: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<SseReq>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, Json<Error>> {
    let pipeline = app.processor.clone();
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
        .inner_join(entity::chunk::Entity)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    let initial_event = if let Some(last_message) = last_message {
        let last_kind = match last_message.kind {
            entity::MessageKind::User => LastKind::User,
            _ => LastKind::Assistant,
        };
        let event = SseResp::Version(SseRespVersion {
            // FIXME: change version when revalidate is needed
            version: last_message.id,
            last_kind,
        });
        let event = Event::default().json_data(event).unwrap();
        Some(Ok(event))
    } else {
        None
    };

    let st = stream::iter(initial_event).chain(stream.map(|token| {
        let event = match token {
            Token::Assitant(content) => SseResp::Token(SseRespToken { content }),
            Token::Reasoning(content) => SseResp::Reasoning(SseRespReasoning { content }),
            Token::Tool { name, args, .. } => SseResp::ToolCall(SseRespToolCall { name, args }),
            Token::Complete {
                message_id,
                chunk_ids,
                token,
                cost,
            } => SseResp::Complete(SseRespMessageComplete {
                id: message_id,
                chunk_ids,
                token_count: token,
                cost,
            }),
            Token::ToolResult(content) => SseResp::ToolResult(SseRespToolResult { content }),
            Token::Error(content) => SseResp::Error(SseRespError { content }),
            Token::Title(title) => SseResp::Title(SseRespTitle { title }),
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
