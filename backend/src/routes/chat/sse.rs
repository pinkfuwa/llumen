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

use crate::{AppState, errors::*, middlewares::auth::UserId, sse::Token};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
/// When connect, the respond will be `Last -> [[Token] -> Enc -> UserMessage]`
/// When update the message, the respond will be `Last -> UserMessage(updated) -> [[Token] -> Enc -> UserMessage]`
pub enum SseResp {
    /// When connect to SSE, the first respond will be this
    /// Use this to get old message
    Last(SseRespLast),

    /// token
    Token(SseRespToken),

    /// End of the streaming message
    /// next token will be `Start`
    End(SseRespEnd),

    /// The message sent by user
    UserMessage(SseRespUserMessage),
}
#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespLast {
    pub id: i32,
    pub version: u32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespEnd {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespToken {
    pub text: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespUserMessage {
    pub id: i32,
    pub text: String,
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
                Token::Last(id, version) => SseResp::Last(SseRespLast { id, version }),
                Token::Token(text) => SseResp::Token(SseRespToken { text }),
                Token::End(id) => SseResp::End(SseRespEnd { id }),
                Token::User(id, text) => SseResp::UserMessage(SseRespUserMessage { id, text }),
            })
        })
        .map(|x| Event::default().json_data(JsonUnion::from(x)));
    Ok(Sse::new(st).keep_alive(KeepAlive::new().interval(Duration::from_secs(10))))
}
