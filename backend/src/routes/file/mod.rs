pub mod download;
pub mod upload;

use axum::{Router, extract::DefaultBodyLimit};

const MAX_FILE_SIZE: usize = 1024 * 1024 * 128; // 128MB

pub fn routes() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/upload", axum::routing::post(upload::route))
        .route("/download/{id}", axum::routing::post(download::route))
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE))
}
