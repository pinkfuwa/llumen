use std::sync::Arc;

use anyhow::Context;
use axum::{Extension, Json, extract::State};
use entity::{message, patch::MessageKind, prelude::*};
use migration::Expr;
use sea_orm::{EntityOrSelect, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use tokio::select;
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, openrouter, sse::PublisherKind};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReq {
    pub chat_id: i32,
    pub text: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessageCreateResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessageCreateReq>,
) -> JsonResult<MessageCreateResp> {
    let chat = Chat::find_by_id(req.chat_id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .context("")
        .kind(ErrorKind::ResourceNotFound)?;

    if chat.owner_id != user_id {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

    let model = Model::find_by_id(chat.model_id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .context("")
        .kind(ErrorKind::Internal)?
        .get_config()
        .context("")
        .kind(ErrorKind::Internal)?;

    let mut puber = app
        .sse
        .publish(req.chat_id)
        .await
        .kind(ErrorKind::Internal)?;
    let msg_id = puber
        .user_message(req.text)
        .await
        .kind(ErrorKind::Internal)?;

    let res = Message::find()
        .select()
        .filter(Expr::col(message::Column::ChatId).eq(req.chat_id))
        .order_by_asc(message::Column::Id)
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let messages = res
        .into_iter()
        .filter_map(|x| {
            let Some(content) = x.text else {
                return None;
            };
            match x.kind {
                MessageKind::User => Some(openrouter::Message::User(content)),
                MessageKind::Assistant => Some(openrouter::Message::Assistant(content)),
                MessageKind::System => Some(openrouter::Message::System(content)),
                MessageKind::Reasoning => None,
            }
        })
        .collect();

    puber.new_stream(PublisherKind::Assistant).await;
    tokio::spawn(async move {
        let res = app
            .openrouter
            .stream(messages, model.into(), Vec::default())
            .await;

        let mut completion = match res {
            Ok(v) => v,
            Err(e) => {
                puber.raw_token(Err(Error {
                    error: ErrorKind::ApiFail,
                    reason: e.to_string(),
                }));
                puber.close().await;

                return;
            }
        };

        loop {
            select! {
                _ = puber.on_halt() => {
                    puber.close().await;
                    completion.close();
                    return;
                }

                token = completion.next() => {
                    match token {
                        Some(Ok(openrouter::StreamCompletionResp::ResponseToken(t))) => {
                            puber.token(&t).await;
                        }
                        Some(Err(e)) => {
                            puber.raw_token(Err(Error {
                                error: ErrorKind::ApiFail,
                                reason: e.to_string(),
                            }));

                            puber.close().await;
                            completion.close();
                            return;
                        }
                        Some(_)=> {
                            continue;
                        }
                        None => {
                            puber.close().await;
                            return;
                        }
                    }
                }
            }
        }
    });

    Ok(Json(MessageCreateResp { id: msg_id }))
}
