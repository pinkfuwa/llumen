use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelListResp {
    pub list: Vec<ModelList>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelList {
    pub id: i32,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelListReq {}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(_): Json<ModelListReq>,
) -> JsonResult<ModelListResp> {
    let models = model::Entity::find()
        .all(&app.conn)
        .await
        .kind(ErrorKind::Internal)?;
    let list = models
        .into_iter()
        .filter_map(|m| {
            Some(ModelList {
                id: m.id,
                display_name: m.get_config()?.display_name,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(ModelListResp { list }))
}
