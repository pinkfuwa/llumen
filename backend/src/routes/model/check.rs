use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelCheckReq {
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelCheckResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelCheckReq>,
) -> JsonResult<ModelCheckResp> {
    let config = req.config;

    let check = model::Model::check_config(&config);

    Ok(Json(ModelCheckResp {
        reason: check.err(),
    }))
}
