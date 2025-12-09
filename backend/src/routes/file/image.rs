use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use entity::file::{self, Entity as File};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::AppState;
use crate::errors::{AppError, Error, ErrorKind, WithKind};
use crate::middlewares::auth::UserId;
use crate::utils::webp::image_to_webp;

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((width, id)): Path<(u32, i32)>,
) -> Result<Response, AppError> {
    let file = File::find_by_id(id)
        .filter(file::Column::OwnerId.eq(user_id))
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "File not found".to_owned(),
        }))?;

    let mime_type = file.mime_type.as_deref().unwrap_or("");
    if !mime_type.starts_with("image/") {
        return Err(Json(Error {
            error: ErrorKind::MalformedRequest,
            reason: "File is not an image".to_owned(),
        })
        .into());
    }

    let original_data = app.blob.get_vectored(id).await.ok_or(Json(Error {
        error: ErrorKind::ResourceNotFound,
        reason: "File data not found".to_owned(),
    }))?;

    let mut content_type = mime_type.to_owned();
    let compressed_data = image_to_webp(&mut content_type, &original_data, width)
        .await
        .map_err(|e| {
            Json(Error {
                error: ErrorKind::Internal,
                reason: format!("Failed to process image: {}", e),
            })
        })?;

    let mut headers = axum::http::HeaderMap::new();

    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_str(&content_type).unwrap(),
    );

    headers.insert(
        axum::http::header::CONTENT_LENGTH,
        axum::http::HeaderValue::from_str(&compressed_data.len().to_string()).unwrap(),
    );

    headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("public, max-age=259200, immutable"),
    );

    headers.insert(
        axum::http::header::ETAG,
        axum::http::HeaderValue::from_str(&format!("\"{}-{}\"", id, width)).unwrap(),
    );

    Ok((headers, compressed_data).into_response())
}
