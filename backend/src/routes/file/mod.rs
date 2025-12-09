pub mod download;
pub mod image;
pub mod upload;

use axum::{Router, extract::DefaultBodyLimit};

const MAX_FILE_SIZE: usize = 1024 * 1024 * 128; // 128MB

pub fn routes() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/upload", axum::routing::post(upload::route))
        .route("/read/{id}", axum::routing::get(download::route))
        .route("/image/{width}/{id}", axum::routing::get(image::route))
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE))
}
