use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::model;
use protocol::{ModelConfig, OcrEngine, WebTool};
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
    pub deep_research: bool,
    pub media_gen: bool,
    pub search_enabled: bool,
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

        let is_custom = app.openrouter.is_custom_api();
        let web = caps.web;
        let user_set_web = config.capability.web.is_some();
        let effective_web = if user_set_web {
            web
        } else if is_custom {
            WebTool::Native
        } else {
            WebTool::OpenRouter
        };
        let search_enabled = caps.toolcall && effective_web != WebTool::Disabled;

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
            media_gen: (config.media_gen.image_model.is_some()
                || config.media_gen.video_model.is_some())
                && caps.toolcall,
            deep_research: cfg!(feature = "deep-research") && caps.toolcall,
            display_name: config.display_name,
            search_enabled,
        });
    }

    Ok(Json(ModelListResp { list }))
}
