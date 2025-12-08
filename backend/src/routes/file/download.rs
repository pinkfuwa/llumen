use axum::response::{IntoResponse, Response};

use crate::errors::AppError;

use bytes::Bytes;

pub async fn route() -> Result<Response, AppError> {
    let data = include_bytes!("../../../image.png");

    let data = Bytes::copy_from_slice(data);

    Ok(data.into_response())
}
