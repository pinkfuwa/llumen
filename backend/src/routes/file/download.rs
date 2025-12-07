use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use entity::file::{self, Entity as File};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::AppState;
use crate::errors::{AppError, Error, ErrorKind, WithKind};
use crate::middlewares::auth::UserId;
use crate::utils::blob::Reader;

use bytes::Bytes;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

struct MmapStream {
    reader: Reader,
    position: usize,
}

impl MmapStream {
    fn new(reader: Reader) -> Self {
        Self {
            reader,
            position: 0,
        }
    }
}

impl Stream for MmapStream {
    type Item = Result<Bytes, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let data = self.reader.as_ref();
        if self.position >= data.len() {
            return Poll::Ready(None);
        }

        // feat: mmap is blocking, we will fix that later
        let end = std::cmp::min(self.position + CHUNK_SIZE, data.len());
        let chunk = Bytes::copy_from_slice(&data[self.position..end]);
        self.position = end;

        Poll::Ready(Some(Ok(chunk)))
    }
}

pub async fn route(
    State(app): State<Arc<AppState>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(id): Path<i32>,
) -> Result<Response, AppError> {
    let file = File::find_by_id(id)
        .filter(file::Column::OwnerId.eq(user_id))
        .one(&app.conn)
        .await
        .kind(ErrorKind::Internal)?
        .ok_or(Json(Error {
            error: ErrorKind::ResourceNotFound,
            reason: "".to_owned(),
        }))?;

    let reader = app.blob.get(id).ok_or(Json(Error {
        error: ErrorKind::ResourceNotFound,
        reason: "File data not found".to_owned(),
    }))?;

    let content_length = reader.as_ref().len();

    let mut headers = axum::http::HeaderMap::new();

    if let Some(mime) = file.mime_type {
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(mime.as_str()).unwrap(),
        );
    }

    headers.insert(
        axum::http::header::CONTENT_LENGTH,
        axum::http::HeaderValue::from_str(&content_length.to_string()).unwrap(),
    );

    let stream = MmapStream::new(reader);
    let body = axum::body::Body::from_stream(stream);

    Ok((headers, body).into_response())
}
