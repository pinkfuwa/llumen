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

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    utils::sse::{SubscribeMessage, SubscribeToken},
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
    /// If no streaming message
    /// use this to get old message
    Last(i32),

    /// start a streaming
    Start(SseRespStart),

    /// token
    Token(String),

    /// End of the streaming message
    /// next token will be `Start`
    End,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct SseRespStart {
    /// Chat room id
    pub id: i32,
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
                    SubscribeToken::Start(id) => SseResp::Start(SseRespStart { id }),
                    SubscribeToken::Token(s) => SseResp::Token(s),
                    SubscribeToken::End => SseResp::End,
                })
                .map_err(|e| Error {
                    error: ErrorKind::Internal,
                    reason: e.to_string(),
                }),
            channel,
        ))
    };
    let remain = stream::unfold(sub.channel, remain);

    let sse = match sub.message {
        SubscribeMessage::Cache(cache) => {
            let start: Result<_, Error> = Ok(SseResp::Start(SseRespStart {
                id: cache.message_id,
            }));
            let token: Result<_, Error> = Ok(SseResp::Token(cache.message));

            stream::iter([start, token]).chain(remain).left_stream()
        }
        SubscribeMessage::Last(id) => {
            let last: Result<_, Error> = Ok(SseResp::Last(id));
            stream::iter([last]).chain(remain).right_stream()
        }
    }
    .map(|x| Event::default().json_data(UResult::from(x)));

    Ok(Sse::new(sse).keep_alive(KeepAlive::new()))
}
