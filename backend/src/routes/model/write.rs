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
    let config = req.config;

    let display_name = <ModelConfig as ModelChecker>::from_toml(&config)
        .map_err(|e| {
            Json(Error {
                error: ErrorKind::MalformedRequest,
                reason: e.to_string(),
            })
        })?
        .display_name;

    let result = model::Entity::update_many()
        .col_expr(model::Column::Config, config.into())
        .filter(model::Column::Id.eq(req.id))
        .exec(&app.conn)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    let wrote = result.rows_affected > 0;

    Ok(Json(ModelWriteResp {
        display_name,
        wrote,
    }))
}
