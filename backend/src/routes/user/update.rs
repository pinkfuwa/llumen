use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::prelude::*;
use protocol::UserPreference;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, TransactionTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct UserUpdateReq {
    /// If omit will use the current user instead
    pub user_id: Option<i32>,
    pub preference: Option<UserPreference>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct UserUpdateResp {
    pub user_id: i32,
}

pub async fn route(
    State(_app): State<Arc<AppState>>,
    Extension(UserId(_user_id)): Extension<UserId>,
    Json(_req): Json<UserUpdateReq>,
) -> JsonResult<UserUpdateResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
