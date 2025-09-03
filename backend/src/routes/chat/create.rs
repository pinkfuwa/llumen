use std::sync::Arc;

use anyhow::Context;
use axum::{Extension, Json, extract::State};
use entity::{MessageKind, chat, message, prelude::*};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    prompts::{self, PromptStore},
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatCreateReq {
    pub model_id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatCreateResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatCreateReq>,
) -> JsonResult<ChatCreateResp> {
    let res = Chat::insert(chat::ActiveModel {
        owner_id: Set(user_id),
        model_id: Set(req.model_id),
        // FIXME:
        // change to auto-gen title
        title: Set("New Chat".to_owned()),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?;

    let user = User::find_by_id(user_id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .context("Cannot find user")
        .kind(ErrorKind::ResourceNotFound)?;

    let template = prompts::ChatStore
        .template(user.preference.locale.as_deref())
        .await
        .kind(ErrorKind::Internal)?
        .render(&app.prompt, res.last_insert_id, (), ())
        .await
        .kind(ErrorKind::Internal)?;

    Message::insert(message::ActiveModel {
        chat_id: Set(res.last_insert_id),
        text: Set(Some(template)),
        kind: Set(MessageKind::System),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?;

    Ok(Json(ChatCreateResp {
        id: res.last_insert_id,
    }))
}
