use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use protocol::{ModelConfig, OcrEngine};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId, openrouter, utils::model::ModelChecker};

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
    pub ocr_file_input: bool,
    pub image_input: bool,
    pub audio_input: bool,
    pub video_input: bool,
    pub native_file_input: bool,
    pub tool: bool,
    pub media_gen: bool,
    pub media_mode_supported: bool,
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

    let mut list = Vec::new();
    for m in models {
        let config =
            <ModelConfig as ModelChecker>::from_toml(&m.config).expect("corruptted database");

        let model: openrouter::Model = config.clone().into();

        let caps = app.openrouter.get_capability(&model).await;

        list.push(ModelList {
            id: m.id,
            image_input: caps.image_input,
            audio_input: caps.audio,
            video_input: caps.video_input,
            ocr_file_input: matches!(
                caps.ocr,
                OcrEngine::Mistral | OcrEngine::Text | OcrEngine::Cloudflare
            ),
            native_file_input: caps.ocr == OcrEngine::Native,
            tool: cfg!(feature = "deep-research") && caps.toolcall,
            media_gen: config.media_gen.image_model.is_some()
                || config.media_gen.video_model.is_some(),
            media_mode_supported: cfg!(feature = "deep-research")
                && caps.toolcall
                && (config.media_gen.image_model.is_some()
                    || config.media_gen.video_model.is_some()),
            display_name: config.display_name,
        });
    }

    Ok(Json(ModelListResp { list }))
}
