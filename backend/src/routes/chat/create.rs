use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{chat, prelude::*};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

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
        title: Set("Test Title".to_owned()),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?;

    Ok(Json(ChatCreateResp {
        id: res.last_insert_id,
    }))
}
