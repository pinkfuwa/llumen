use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelDeleteReq {
    pub id: i32,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelDeleteResp {
    pub deleted: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelDeleteReq>,
) -> JsonResult<ModelDeleteResp> {
    model::Entity::delete_by_id(req.id)
        .exec(&app.conn)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    Ok(Json(ModelDeleteResp { deleted: true }))
}
