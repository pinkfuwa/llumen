use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{message, patch::MessageKind, prelude::*};
use migration::Expr;
use sea_orm::{EntityOrSelect, EntityTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use tokio::select;
use typeshare::typeshare;

use crate::{
    AppState, errors::*, middlewares::auth::UserId, openrouter::chat_completions,
    sse::PublisherKind,
};

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
    let res = Chat::find_by_id(req.chat_id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    if !res.is_some_and(|x| x.owner_id == user_id) {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

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
        .expr(Expr::col(message::Column::ChatId).eq(req.chat_id))
        .order_by_asc(message::Column::Id)
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let messages = res
        .into_iter()
        .filter_map(|x| {
            let role = match x.kind {
                MessageKind::User => chat_completions::Role::User,
                MessageKind::Assistant => chat_completions::Role::Assistant,
                MessageKind::Reasoning => return None,
            };
            let Some(content) = x.text else {
                return None;
            };
            Some(chat_completions::Message { role, content })
        })
        .collect();

    puber.new_stream(PublisherKind::Assistant).await;
    tokio::spawn(async move {
        let res = chat_completions::Completion::request(
            messages,
            "openai/gpt-oss-20b:free".to_owned(),
            &app.api_key,
        )
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

                c = completion.next() => {
                    match c {
                        Some(Ok(v)) => {
                            if v.choices.is_empty() {
                                continue;
                            };

                            puber.token(&v.choices[0].delta.content).await;
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
