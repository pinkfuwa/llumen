use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{message, patch::MessageKind, prelude::*};
use migration::Expr;
use sea_orm::{ActiveValue::Set, EntityOrSelect, EntityTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use tokio::select;
use typeshare::typeshare;

use crate::{
    AppState, errors::*, middlewares::auth::UserId, openrouter::chat_completions,
    utils::sse::PublishToken,
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

    let publish = app
        .sse
        .publish(req.chat_id)
        .await
        .kind(ErrorKind::Internal)?;
    let user_msg_res = Message::insert(message::ActiveModel {
        chat_id: Set(req.chat_id),
        text: Set(req.text.clone()),
        kind: Set(MessageKind::User),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?;
    publish.channel.send(PublishToken::UserText(
        user_msg_res.last_insert_id,
        req.text,
    ));
    let res = Message::find()
        .select()
        .expr(Expr::col(message::Column::ChatId).eq(req.chat_id))
        .order_by_desc(message::Column::Id)
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let messages = res
        .into_iter()
        .filter_map(|x| {
            let role = match x.kind {
                MessageKind::User => chat_completions::Role::User,
                MessageKind::Assistant => chat_completions::Role::Assistant,
                MessageKind::Think => return None,
            };
            Some(chat_completions::Message {
                role,
                content: x.text,
            })
        })
        .collect();
    tokio::spawn(async move {
        let res = chat_completions::Completion::request(
            messages,
            "google/gemma-3-4b-it:free".to_owned(),
            &app.api_key,
        )
        .await;

        let mut completion = match res {
            Ok(v) => v,
            Err(e) => {
                publish.channel.send(PublishToken::Error(Error {
                    error: ErrorKind::ApiFail,
                    reason: e.to_string(),
                }));

                return;
            }
        };

        loop {
            select! {
                _ = publish.halt.notified() => {
                    publish
                        .channel
                        .send(PublishToken::End);
                    return;
                }

                c = completion.next() => {
                    match c {
                        Some(Ok(mut v)) => {
                            if v.choices.is_empty() {
                                continue;
                            };

                            publish
                                .channel
                                .send(PublishToken::Text(v.choices.swap_remove(0).delta.content));
                        }
                        Some(Err(e)) => {
                            publish
                                .channel
                                .send(PublishToken::Error(Error {
                                    error: ErrorKind::ApiFail,
                                    reason: e.to_string(),
                                }));
                            completion.close();
                        }
                        None => {
                            publish
                                .channel
                                .send(PublishToken::End);
                            return;
                        }
                    }
                }
            }
        }
    });

    Ok(Json(MessageCreateResp {
        id: user_msg_res.last_insert_id,
    }))
}
