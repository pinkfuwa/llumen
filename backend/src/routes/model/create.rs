use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{model, prelude::*};
use protocol::ModelConfig;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    errors::*,
    middlewares::auth::UserId,
    utils::model::{ModelCapability, ModelChecker},
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ModelCreateReq {
    pub config: String,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct ModelCreateResp {
    pub id: i32,
    pub display_name: String,
    pub image_input: bool,
    pub audio_input: bool,
    pub other_file_input: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelCreateReq>,
) -> JsonResult<ModelCreateResp> {
    let config = req.config;

    match <ModelConfig as ModelChecker>::from_toml(&config) {
        Ok(cfg) => {
            let id = Model::insert(model::ActiveModel {
                config: Set(config),
                ..Default::default()
            })
            .exec(&app.conn)
            .await
            .kind(ErrorKind::Internal)?
            .last_insert_id;

            Ok(Json(ModelCreateResp {
                id,
                image_input: cfg.is_image_capable(),
                audio_input: cfg.is_audio_capable(),
                other_file_input: cfg.is_other_file_capable(),
                display_name: cfg.display_name,
            }))
        }
        Err(reason) => Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: reason.to_string(),
        })),
    }
}
