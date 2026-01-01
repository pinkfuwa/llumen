use std::sync::Arc;

use axum::{Extension, Json, extract::State};
use entity::file::{Column as FileColumn, Entity as File};
use protocol::{FileMetadata, MessageInner};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{
    AppState,
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
    let file_ids: Vec<i32> = req.files.iter().map(|f| f.id).collect();

    if !file_ids.is_empty() {
        File::update_many()
            .filter(FileColumn::Id.is_in(file_ids.clone()))
            .filter(FileColumn::OwnerId.eq(user_id))
            .col_expr(
                FileColumn::ChatId,
                sea_orm::sea_query::Expr::value(req.chat_id),
            )
            .col_expr(
                FileColumn::ValidUntil,
                sea_orm::sea_query::Expr::value(Option::<i32>::None),
            )
            .exec(&app.conn)
            .await
            .raw_kind(ErrorKind::Internal)?;
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
