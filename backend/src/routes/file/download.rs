use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use entity::file::{self, Entity as File};
use futures_util::FutureExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio::task::{JoinHandle, spawn_blocking};

use crate::AppState;
use crate::errors::{AppError, Error, ErrorKind, WithKind};
use crate::middlewares::auth::UserId;
use crate::utils::blob::Reader;

use bytes::Bytes;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

const CHUNK_SIZE: usize = 256 * 1024; // 256KB chunks

struct MmapStream {
    reader: Arc<Reader>,
    position: usize,
    read_task: Option<JoinHandle<Bytes>>,
}

impl MmapStream {
    fn new(reader: Reader) -> Self {
        Self {
            reader: Arc::new(reader),
            position: 0,
            read_task: None,
        }
    }
}

impl Stream for MmapStream {
    type Item = Result<Bytes, axum::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.position >= self.reader.len() {
            return Poll::Ready(None);
        }

        let position = self.position;
        let end = std::cmp::min(position + CHUNK_SIZE, self.reader.len());

        if self.read_task.is_none() {
            let reader = self.reader.clone();
            self.read_task = Some(spawn_blocking(move || {
                Bytes::copy_from_slice(&reader.as_ref().as_ref()[position..end])
            }));
        }

        self.read_task
            .as_mut()
            .unwrap()
            .poll_unpin(_cx)
            .map(|x| match x {
                Ok(buf) => {
                    self.position = end;
                    self.read_task = None;
                    Some(Ok(buf))
                }
                Err(_) => None,
            })
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
