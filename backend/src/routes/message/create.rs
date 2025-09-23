use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::MessageKind;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    chat::{Normal, Pipeline, Search},
    errors::*,
    middlewares::auth::UserId,
    utils::chat::ChatMode,
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReq {
    pub chat_id: i32,
    pub model_id: i32,
    pub mode: ChatMode,
    pub text: String,
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

    let closure = async move {
        let mut completion_ctx = app
            .processor
            .get_completion_context(user_id, req.chat_id, req.model_id)
            .await?;

        completion_ctx.set_mode(req.mode.into());

        match req.mode {
            ChatMode::Search => Search::process(app.processor.clone(), completion_ctx).await?,
            _ => Normal::process(app.processor.clone(), completion_ctx).await?,
        };

        Ok::<(), anyhow::Error>(())
    };
    tokio::spawn(async move {
        if let Err(e) = closure.await {
            tracing::error!("Failed to process message: {:?}", e);
        }
    });

    Ok(Json(MessageCreateResp { id: user_msg.id }))
}
