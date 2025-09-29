use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Multipart, State};
use entity::file::Entity as File;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::Serialize;
use typeshare::typeshare;

use crate::{AppState, errors::*, middlewares::auth::UserId};

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
    // https://docs.rs/multer/3.1.0/multer/struct.Multipart.html#field-exclusivity
    // > That is, a Field emitted by next_field() must be dropped before calling next_field() again.
    // > Failure to do so will result in an error.
    let chat_id = {
        let chat_field = multipart
            .next_field()
            .await
            .kind(ErrorKind::MalformedRequest)?;

        if chat_field.is_none() {
            return Err(Json(Error {
                error: ErrorKind::MalformedRequest,
                reason: "missing chat_id field".into(),
            }));
        }

        let chat_field = chat_field.unwrap();
        chat_field
            .text()
            .await
            .kind(ErrorKind::MalformedRequest)?
            .trim()
            .parse()
            .kind(ErrorKind::MalformedRequest)?
    };

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
            reason: "chat_id must be sent before the file part".into(),
        }));
    }

    let content_field = content_field.unwrap();

    let mime_type = content_field.content_type().map(|c| c.to_string());

    let data = content_field
        .bytes()
        .await
        .kind(ErrorKind::MalformedRequest)?;

    let file = File::insert(entity::file::ActiveModel {
        chat_id: Set(Some(chat_id)),
        owner_id: Set(Some(user_id)),
        mime_type: Set(mime_type),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?
    .last_insert_id;

    app.blob.insert(file, data.to_vec()).unwrap();

    Ok(Json(FileUploadResp { id: file }))
}
