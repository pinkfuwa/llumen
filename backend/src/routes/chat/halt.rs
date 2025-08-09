use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ChatHaltReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ChatHaltResp {}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ChatHaltReq>,
) -> JsonResult<ChatHaltResp> {
    let res = Chat::find_by_id(req.id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    if res.is_none_or(|x| x.owner_id != user_id) {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

    app.sse.halt(req.id).kind(ErrorKind::Internal)?;
    Ok(Json(ChatHaltResp {}))
}
