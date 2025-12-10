use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{prelude::*, user};
use sea_orm::{ActiveValue, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserCreateReq {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserCreateResp {
    pub user_id: i32,
}

pub async fn route(
    State(_app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_req): Json<UserCreateReq>,
) -> JsonResult<UserCreateResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
