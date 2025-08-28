use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{model, patch::ModelConfig};
use sea_orm::EntityTrait;
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
pub struct ModelReadResp(ModelConfig);

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelReadReq>,
) -> JsonResult<ModelReadResp> {
    let model = model::Entity::find_by_id(req.id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;

    let model = model.ok_or_else(|| Error {
        error: ErrorKind::ResourceNotFound,
        reason: "model not found".to_owned(),
    })?;

    let config = model.get_config().ok_or_else(|| Error {
        error: ErrorKind::Internal,
        reason: "invalid model config".to_owned(),
    })?;

    Ok(Json(ModelReadResp(config)))
}
