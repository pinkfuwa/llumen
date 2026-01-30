use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{model, prelude::*};
use protocol::{ModelConfig, OcrEngine};
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState, errors::*, middlewares::auth::UserId, openrouter, utils::model::ModelChecker,
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
    pub tool: bool,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(_)): Extension<UserId>,
    Json(req): Json<ModelCreateReq>,
) -> JsonResult<ModelCreateResp> {
    let raw_config = req.config;

    match <ModelConfig as ModelChecker>::from_toml(&raw_config) {
        Ok(config) => {
            let model: openrouter::Model = config.clone().into();
            let caps = app.processor.get_capability(&model).await;

            let id = Model::insert(model::ActiveModel {
                config: Set(raw_config),
                ..Default::default()
            })
            .exec(&app.conn)
            .await
            .kind(ErrorKind::Internal)?
            .last_insert_id;

            Ok(Json(ModelCreateResp {
                id,
                image_input: caps.image_input,
                audio_input: caps.audio,
                other_file_input: caps.ocr != OcrEngine::Disabled,
                tool: caps.toolcall,
                display_name: config.display_name,
            }))
        }
        Err(reason) => Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: reason.to_string(),
        })),
    }
}
