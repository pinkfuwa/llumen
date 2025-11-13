use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use protocol::ModelConfig;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    utils::model::{ModelCapability, ModelChecker},
};

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
    pub image_input: bool,
    pub audio_input: bool,
    pub other_file_input: bool,
    pub tool: bool,
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
            let config =
                <ModelConfig as ModelChecker>::from_toml(&m.config).expect("corruptted database");
            Some(ModelList {
                id: m.id,
                image_input: config.is_image_capable(),
                audio_input: config.is_audio_capable(),
                other_file_input: config.is_other_file_capable(),
                tool: config.is_tool_capable(),
                display_name: config.display_name,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(ModelListResp { list }))
}
