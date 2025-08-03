use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelReadReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelReadResp {
    pub name: String,
    pub capability: ModelReadRespCapability,
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelReadRespCapability {
    pub image: bool,
    pub audio: bool,
    pub ocr: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ModelReadReq>,
) -> JsonResult<ModelReadResp> {
    todo!()
}
