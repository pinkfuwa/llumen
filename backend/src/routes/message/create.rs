use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::MessageKind;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{self, Set},
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    chat::{NormalPipeline, Pipeline},
    errors::*,
    middlewares::auth::UserId,
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReq {
    pub chat_id: i32,
    pub mode: MessageCreateReqMode,
    pub text: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[typeshare]
#[serde(rename_all = "snake_case")]
pub enum MessageCreateReqMode {
    Normal,
    Search,
    Agent,
    Research,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessageCreateResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<MessageCreateReq>,
) -> JsonResult<MessageCreateResp> {
    let txn = app.conn.begin().await.raw_kind(ErrorKind::Internal)?;
    let user_msg = entity::message::ActiveModel {
        chat_id: Set(req.chat_id),
        kind: Set(MessageKind::User),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .raw_kind(ErrorKind::Internal)?;
    entity::chunk::ActiveModel {
        message_id: Set(user_msg.id),
        content: Set(req.text),
        kind: Set(entity::ChunkKind::Text),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .raw_kind(ErrorKind::Internal)?;
    txn.commit().await.raw_kind(ErrorKind::Internal)?;

    let completion_ctx = app
        .pipeline
        .get_completion_context(user_id, req.chat_id)
        .await
        .raw_kind(ErrorKind::Internal)?;

    let msg_id = completion_ctx.get_message_id();

    NormalPipeline::process(app.pipeline.clone(), completion_ctx)
        .await
        .raw_kind(ErrorKind::Internal)?;

    Ok(Json(MessageCreateResp { id: msg_id }))
}
