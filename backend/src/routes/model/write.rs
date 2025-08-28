use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelWriteReq {
    pub id: i32,
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelWriteResp {}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<ModelWriteReq>,
) -> JsonResult<ModelWriteResp> {
    let config = req.config;

    if let Err(reason) = model::Model::check_config(&config) {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason,
        }));
    }

    model::Entity::update_many()
        .col_expr(model::Column::Config, config.into())
        .filter(model::Column::Id.eq(req.id))
        .exec(&app.conn)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    Ok(Json(ModelWriteResp {}))
}
