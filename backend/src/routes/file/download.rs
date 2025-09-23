use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use entity::file::Entity as File;
use sea_orm::EntityTrait;

use crate::AppState;
use crate::errors::{AppError, Error, ErrorKind, JsonResult, WithKind};

pub async fn route(
    State(app): State<Arc<AppState>>,
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

    let blob = app.blob.get(id).await.unwrap();

    let mut headers = axum::http::HeaderMap::new();

    if let Some(mime) = file.mime_type {
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(mime.as_str()).unwrap(),
        );
    }

    Ok((headers, Bytes::from(blob.to_vec())).into_response())
}
