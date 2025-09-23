use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::Json;
use entity::file::Entity as File;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::Serialize;
use typeshare::typeshare;

use crate::{errors::*, AppState};

#[derive(Debug, Serialize)]
#[typeshare]
pub struct FileUploadResp {
    pub id: i32,
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> JsonResult<FileUploadResp> {
    let chat_id = multipart
        .next_field()
        .await
        .unwrap()
        .unwrap()
        .text()
        .await
        .unwrap()
        .parse::<i32>()
        .unwrap();

    let field = multipart.next_field().await.unwrap().unwrap();

    let filename = field.file_name().unwrap().to_string();
    let content_type = field.content_type().map(|x| x.to_string());
    let data = field.bytes().await.unwrap().to_vec();

    let file = File::insert(entity::file::ActiveModel {
        chat_id: Set(chat_id),
        filename: Set(filename),
        mime_type: Set(content_type),
        ..Default::default()
    })
    .exec(&app.conn)
    .await
    .kind(ErrorKind::Internal)?
    .last_insert_id;

    app.blob.insert(file, data).unwrap();

    Ok(Json(FileUploadResp { id: file }))
}
