use std::sync::Arc;

use crate::AppState;
use axum::{
    Router,
    body::Body,
    extract::Json,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use typeshare::typeshare;

#[derive(Debug, Deserialize)]
#[typeshare]
pub struct ImageProxyRequest {
    pub url: String,
}

// FIXME: the proxy have auth issue, consider running javascript fetch with credentials: 'include'
async fn image_proxy(Json(req): Json<ImageProxyRequest>) -> Result<Response, StatusCode> {
    let url = req.url;

    if !(url.ends_with("ico")
        || url.ends_with("png")
        || url.ends_with("jpg")
        || url.ends_with("svg")
        || url.ends_with("webp"))
    {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    if !content_type.starts_with("image/") {
        return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    let mut headers = HeaderMap::new();
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        headers.insert(header::CONTENT_TYPE, content_type.clone());
    }

    let body = Body::from_stream(response.bytes_stream());

    Ok((headers, body).into_response())
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", post(image_proxy))
}
