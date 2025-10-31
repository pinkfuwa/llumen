use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::{ChunkKind, MessageKind, chunk, patch::FileHandle};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json;
use typeshare::typeshare;

use crate::{
    AppState,
    chat::{Normal, Pipeline, Search},
    errors::{ErrorKind, JsonResult, WithKind},
    middlewares::auth::UserId,
    utils::chat::ChatMode,
};

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReqFile {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct MessageCreateReq {
    pub chat_id: i32,
    pub model_id: i32,
    pub mode: ChatMode,
    pub text: String,
    pub files: Vec<MessageCreateReqFile>,
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct MessageCreateResp {
    pub id: i32,
    pub user_id: i32,
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

    let mut chunks = vec![chunk::ActiveModel {
        message_id: Set(user_msg.id),
        content: Set(req.text),
        kind: Set(ChunkKind::Text),
        ..Default::default()
    }];
    chunks.extend(req.files.into_iter().map(|f| {
        let file_handle = FileHandle {
            name: f.name,
            id: f.id,
        };
        chunk::ActiveModel {
            message_id: Set(user_msg.id),
            content: Set(serde_json::to_string(&file_handle).unwrap()),
            kind: Set(ChunkKind::File),
            ..Default::default()
        }
    }));

    chunk::Entity::insert_many(chunks)
        .exec(&txn)
        .await
        .raw_kind(ErrorKind::Internal)?;

    txn.commit().await.raw_kind(ErrorKind::Internal)?;

    let mut completion_ctx = app
        .processor
        .get_completion_context(user_id, req.chat_id, req.model_id)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    let id = completion_ctx.get_message_id();

    let closure = async move {
        completion_ctx.set_mode(req.mode.into());

        match req.mode {
            ChatMode::Search => Search::process(app.processor.clone(), completion_ctx).await?,
            _ => Normal::process(app.processor.clone(), completion_ctx).await?,
        };

        Ok::<(), anyhow::Error>(())
    };

    tokio::spawn(async move {
        if let Err(e) = closure.await {
            log::error!("Failed to process message: {:?}", e);
        }
    });

    Ok(Json(MessageCreateResp {
        user_id: user_msg.id,
        id,
    }))
}
