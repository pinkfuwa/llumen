use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use protocol::ModelConfig;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, utils::model::ModelChecker};

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
    State(_): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelCheckReq>,
) -> JsonResult<ModelCheckResp> {
    let config = req.config;

    Ok(Json(ModelCheckResp {
        reason: <ModelConfig as ModelChecker>::from_toml(&config)
            .err()
            .map(|e| e.to_string()),
    }))
}
