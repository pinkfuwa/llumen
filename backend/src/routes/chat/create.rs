use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{chat, prelude::*};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, utils::chat::ChatMode};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatCreateReq {
    pub model_id: i32,
    pub mode: ChatMode,
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
    let chat_id = Chat::insert(chat::ActiveModel {
        owner_id: Set(user_id),
        model_id: Set(req.model_id),
        title: Set(None),
        mode: Set(req.mode.into()),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?
    .last_insert_id;

    Ok(Json(ChatCreateResp { id: chat_id }))
}
