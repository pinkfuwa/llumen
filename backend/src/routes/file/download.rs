use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use entity::chat;
use entity::file::Entity as File;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::AppState;
use crate::errors::{AppError, Error, ErrorKind, WithKind};
use crate::middlewares::auth::UserId;

use bytes::Bytes;

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(id): Path<i32>,
) -> Result<Response, AppError> {
    let file = File::find_by_id(id)
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }))?;

    // Check access: user must own the file OR own the chat the file belongs to
    let has_access = if let Some(owner_id) = file.owner_id {
        owner_id == user_id
    } else if let Some(chat_id) = file.chat_id {
        // Check if user owns the chat
        chat::Entity::find_by_id(chat_id)
            .filter(chat::Column::OwnerId.eq(user_id))
            .one(&app.conn)
            .await
            .kind(ErrorKind::Internal)?
            .is_some()
    } else {
        false
    };

    if !has_access {
        return Err(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }));
    }

    let blob = app.blob.get_vectored(id).await.unwrap();

    let mut headers = axum::http::HeaderMap::new();

    if let Some(mime) = file.mime_type {
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(mime.as_str()).unwrap(),
        );
    }

    Ok((headers, Bytes::from(blob)).into_response())
}
