use std::sync::Arc;

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
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, utils::sse::SubscribeToken};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct SseReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
#[serde(tag = "t", content = "c", rename_all = "snake_case")]
pub enum SseResp {
    /// If no streaming message
    /// use this to get old message
    Last(SseRespLast),

    /// start a streaming
    AssistantStart,

    /// token
    Token(SseRespToken),

    /// End of the streaming message
    /// next token will be `Start`
    End(SseRespEnd),

    UserMessage(SseRespUserMessage),
}
#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespLast {
    pub id: i32,
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

    let remain = |mut channel: broadcast::Receiver<SubscribeToken>| async move {
        Some((
            channel
                .recv()
                .await
                .map(|v| match v {
                    SubscribeToken::UserText(id, text) => {
                        Ok(SseResp::UserMessage(SseRespUserMessage { id, text }))
                    }
                    SubscribeToken::AssistantStart => Ok(SseResp::AssistantStart),
                    SubscribeToken::Token(text) => Ok(SseResp::Token(SseRespToken { text })),
                    SubscribeToken::Error(error) => Err(error),
                    SubscribeToken::End(id) => Ok(SseResp::End(SseRespEnd { id })),
                })
                .map_err(|e| Error {
                    error: ErrorKind::Internal,
                    reason: e.to_string(),
                })
                .and_then(|x| x),
            channel,
        ))
    };
    let remain = stream::unfold(sub.channel, remain);

    let last = Ok(SseResp::Last(SseRespLast { id: sub.last }));
    let sse = match sub.message {
        Some(text) => stream::iter([last, Ok(SseResp::Token(SseRespToken { text }))]).left_stream(),
        None => stream::iter([last]).right_stream(),
    }
    .chain(remain)
    .map(|x| Event::default().json_data(JsonUnion::from(x)));

    Ok(Sse::new(sse).keep_alive(KeepAlive::new()))
}
