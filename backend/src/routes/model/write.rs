use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use protocol::ModelConfig;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, utils::model::ModelChecker};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelWriteReq {
    pub id: i32,
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelWriteResp {
    display_name: String,
    wrote: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelWriteReq>,
) -> JsonResult<ModelWriteResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
