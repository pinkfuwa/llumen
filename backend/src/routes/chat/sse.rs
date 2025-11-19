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
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};
use typeshare::typeshare;

use crate::{
    AppState,
    chat::{Cursor, Token},
    errors::*,
    middlewares::auth::UserId,
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseReqResume {
    pub cursor: SseCursor,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseCursor {
    pub index: i32,
    pub offset: i32,
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseReq {
    pub id: i32,
    pub resume: Option<SseReqResume>,
}

/// Represents a message sent over the SSE (Server-Sent Events) stream in the chat API.
///
/// Each enum variant corresponds to a specific event or data payload that can be emitted to the
/// client during a chat session. The enum is serialized in a tagged form with fields
/// `{ "t": "<variant>", "c": <content> }` (snake_case variant names).
///
/// Concatenation semantics for assembling a complete assistant message:
/// - Assistant-generated text is streamed as a sequence of `Token(String)` events.
/// - The assistant's internal reasoning is streamed as `Reasoning(String)` events.
/// - In "deep" research mode, higher-level plans and final reports are streamed as
///   `DeepPlan(String)` and `DeepReport(String)` respectively, and individual deep-step
///   outputs use `DeepStep*` variants.
///
/// To reconstruct a full, human-facing message the client SHOULD concatenate the textual
/// chunks in the order they are received:
/// - For normal assistant responses: append `Token` chunks (and optionally interleave
///   `Reasoning` chunks if the client wants to surface reasoning). When a `Complete` event
///   arrives it indicates the assistant finished producing the message and provides final
///   metadata (message id, token count, cost, version).
/// - For deep-research messages: concatenate `DeepPlan` (if any), `DeepStepToken` and
///   `DeepStepReasoning` chunks as they arrive, and finally include `DeepReport` when it is
///   emitted. `Complete` is still used to indicate the message is finalized and carries
///   the canonical metadata for the completed message.
///
/// Other variants represent discrete non-textual events:
/// - `Version(i32)`: an initial signal of the latest message/version id for the chat.
/// - `ToolCall(SseRespToolCall)`: a tool invocation with name and args.
/// - `ToolResult(SseRespToolResult)` / `DeepStepToolResult(SseRespToolResult)`: tool outputs.
/// - `Start(SseStart)`: indicates the beginning of processing for a new assistant message.
/// - `Title(String)`: an updated or generated title for the chat.
/// - `Error(String)`: an error message to surface to the client.
///
/// Important: the client should treat text-bearing variants (`Token`, `Reasoning`,
/// `DeepPlan`, `DeepStepToken`, `DeepStepReasoning`, `DeepReport`) as streamable fragments
/// that together form the final content; `Complete` is the canonical signal that final
/// assembly is complete and includes definitive metadata.
#[derive(Debug, Serialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum SseResp {
    Version(i32),
    Token(String),
    Reasoning(String),
    ToolCall(SseRespToolCall),
    ToolResult(SseRespToolResult),
    Complete(SseRespMessageComplete),
    Title(String),
    Error(String),
    Start(SseStart),
    DeepPlan(String),
    DeepStepStart(i32),
    DeepStepToken(String),
    DeepStepReasoning(String),
    DeepStepToolResult(SseRespToolResult),
    DeepStepToolCall(SseRespToolCall),
    DeepReport(String),
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
    pub token_count: i32,
    pub cost: f32,
    pub version: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseStart {
    pub id: i32,
    pub user_msg_id: i32,
    pub version: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<SseReq>,
) -> Result<Sse<impl Stream<Item = Result<Event, axum::Error>>>, Json<Error>> {
    #[cfg(feature = "tracing")]
    {
        use tracing::info;
        info!(
            user_id = user_id,
            chat_id = req.id,
            "subscribing to chat events"
        );
    }

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

    // last non-empty message
    let last_msg = Message::find()
        .filter(entity::message::Column::ChatId.eq(req.id))
        .order_by_desc(entity::message::Column::Id)
        .limit(3)
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .into_iter()
        .find(|m| !m.inner.is_empty());

    let initial_event = if let Some(ref last_msg) = last_msg {
        let event = SseResp::Version(last_msg.id);
        let event = Event::default().json_data(event).unwrap();
        Some(Ok(event))
    } else {
        None
    };

    let cursor = req
        .resume
        .map(|resume| match last_msg.map(|x| x.id) {
            Some(i) if i == resume.version => {
                let SseCursor { offset, index } = resume.cursor;
                Cursor::try_from((index, offset)).ok()
            }
            _ => None,
        })
        .unwrap_or(None);

    let stream = pipeline.clone().subscribe(req.id, cursor);

    let st = stream::iter(initial_event).chain(stream.filter_map(|token| {
        let event = match token {
            Token::Assistant(content) => SseResp::Token(content),
            Token::Reasoning(content) => SseResp::Reasoning(content),
            Token::ToolCall { name, arg } => SseResp::ToolCall(SseRespToolCall { name, args: arg }),
            Token::Complete {
                message_id,
                token,
                cost,
            } => SseResp::Complete(SseRespMessageComplete {
                id: message_id,
                token_count: token,
                cost,
                version: message_id,
            }),
            Token::ToolResult(content) => SseResp::ToolResult(SseRespToolResult { content }),
            Token::Error(content) => SseResp::Error(content),
            Token::Title(title) => SseResp::Title(title),
            Token::Start { id, user_msg_id } => SseResp::Start(SseStart {
                id,
                user_msg_id,
                version: user_msg_id,
            }),
            Token::Empty => return None,
            Token::DeepPlan(content) => SseResp::DeepPlan(content),
            Token::DeepStepStart(content) => SseResp::DeepStepStart(content),
            Token::DeepStepReasoning(content) => SseResp::DeepStepReasoning(content),
            Token::DeepStepToolCall { name, arg } => {
                SseResp::DeepStepToolCall(SseRespToolCall { name, args: arg })
            }
            Token::DeepStepToken(content) => SseResp::DeepStepToken(content),
            Token::DeepStepToolResult(content) => {
                SseResp::DeepStepToolResult(SseRespToolResult { content })
            }
            Token::DeepReport(content) => SseResp::DeepReport(content),
        };

        Some(Ok(Event::default().json_data(event).unwrap()))
    }));

    Ok(Sse::new(st).keep_alive(KeepAlive::default().interval(Duration::from_secs(30))))
}
