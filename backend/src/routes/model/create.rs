use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{model, prelude::*};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelCreateReq {
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelCreateResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelCreateReq>,
) -> JsonResult<ModelCreateResp> {
    let config = req.config;

    if let Err(reason) = model::Model::check_config(&config) {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason,
        }));
    }

    let res = Model::insert(model::ActiveModel {
        config: Set(config),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?;

    Ok(Json(ModelCreateResp {
        id: res.last_insert_id,
    }))
}
