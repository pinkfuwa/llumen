use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelDeleteReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelDeleteResp {
    pub deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelDeleteReq>,
) -> JsonResult<ModelDeleteResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
