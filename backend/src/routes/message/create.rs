use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use protocol::{FileMetadata, MessageInner};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
    errors::{Error, ErrorKind, JsonResult, WithKind},
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
    if req.chat_id == 1 {
        return Err(Json(Error {
            error: ErrorKind::Internal,
            reason: "do not send in tutorial chat".to_string(),
        }));
    }
    let files = req
        .files
        .into_iter()
        .map(|f| FileMetadata {
            name: f.name,
            id: f.id,
        })
        .collect::<Vec<_>>();

    let user_msg = entity::message::ActiveModel {
        chat_id: Set(req.chat_id),
        inner: Set(MessageInner::User {
            text: req.text,
            files,
        }),
        ..Default::default()
    }
    .insert(&app.conn)
    .await
    .raw_kind(ErrorKind::Internal)?;

    let mut completion_ctx = app
        .processor
        .get_completion_context(user_id, req.chat_id, req.model_id)
        .await
        .kind(ErrorKind::ResourceNotFound)?;

    let id = completion_ctx.get_message_id();

    let closure = async move {
        completion_ctx.set_mode(req.mode.into());

        app.processor.clone().process(completion_ctx).await?;

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
