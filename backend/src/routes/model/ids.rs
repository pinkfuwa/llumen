use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelIdsReq {}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelIdsResp {
    pub ids: Vec<String>,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_): Json<ModelIdsReq>,
) -> JsonResult<ModelIdsResp> {
    let ids = app.processor.get_model_ids();
    Ok(Json(ModelIdsResp { ids }))
}
