use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserInfoReq {
    /// If omit will use the current user instead
    pub user_id: Option<i32>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserInfoResp {
    pub user_id: i32,
    pub username: String,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<UserInfoReq>,
) -> JsonResult<UserInfoResp> {
    let user_id = req.user_id.unwrap_or(user_id);

    let res = User::find_by_id(user_id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or("")
        .kind(ErrorKind::ResourceNotFound)?;

    Ok(Json(UserInfoResp {
        user_id: res.id,
        username: res.name,
    }))
}
