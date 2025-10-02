use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::prelude::*;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserDeleteReq {
    pub user_id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserDeleteResp {
    pub deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<UserDeleteReq>,
) -> JsonResult<UserDeleteResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
