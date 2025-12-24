use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{model, prelude::*};
use protocol::{ModelConfig, OcrEngine};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState, errors::*, middlewares::auth::UserId, openrouter, utils::model::ModelChecker,
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelCreateReq {
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelCreateResp {
    pub id: i32,
    pub display_name: String,
    pub image_input: bool,
    pub audio_input: bool,
    pub other_file_input: bool,
    pub tool: bool,
}

pub async fn route(
    State(_app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_req): Json<ModelCreateReq>,
) -> JsonResult<ModelCreateResp> {
    Err(Json(Error {
        error: ErrorKind::Internal,
        reason: "not available in demo".to_string(),
    }))
}
