use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Multipart, State};
use entity::file::Entity as File;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::Serialize;
use typeshare::typeshare;

use crate::routes::file::MAX_FILE_SIZE;
use crate::{AppState, errors::*, middlewares::auth::UserId};

fn get_valid_until_timestamp() -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    (now + 3600) as i32
}

#[derive(Debug, Serialize)]
#[typeshare]
pub struct FileUploadResp {
    pub id: i32,
}

const FILE_FIELD: &str = "file";

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    mut multipart: Multipart,
) -> JsonResult<FileUploadResp> {
    let size_field = multipart
        .next_field()
        .await
        .kind(ErrorKind::MalformedRequest)?;

    if size_field.is_none() {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "missing size field".into(),
        }));
    }

    if size_field.as_ref().unwrap().name() != Some("size") {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "size must be sent before the file part".into(),
        }));
    }

    let size = size_field
        .unwrap()
        .text()
        .await
        .kind(ErrorKind::MalformedRequest)?
        .parse::<i32>()
        .kind(ErrorKind::MalformedRequest)?;

    if size > MAX_FILE_SIZE as i32 {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "file size exceeds the limit".to_string(),
        }));
    }

    let content_field = multipart
        .next_field()
        .await
        .kind(ErrorKind::MalformedRequest)?;

    if content_field.is_none() {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "missing file field".into(),
        }));
    }

    if content_field.as_ref().unwrap().name() != Some(FILE_FIELD) {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "size must be sent before the file part".into(),
        }));
    }

    let content_field = content_field.unwrap();

    let mime_type = content_field.content_type().map(|c| c.to_string());

    let file_id = File::insert(entity::file::ActiveModel {
        chat_id: Set(None),
        owner_id: Set(Some(user_id)),
        mime_type: Set(mime_type),
        valid_until: Set(Some(get_valid_until_timestamp())),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?
    .last_insert_id;

    app.blob
        .insert_with_error(file_id, size.max(0) as usize, content_field)
        .await
        .kind(ErrorKind::Internal)?
        .kind(ErrorKind::MalformedRequest)?;

    Ok(Json(FileUploadResp { id: file_id }))
}
