pub mod download;
pub mod upload;

use axum::Router;

pub fn routes() -> Router<std::sync::Arc<crate::AppState>> {
    Router::new()
        .route("/upload", axum::routing::post(upload::route))
        .route("/{id}", axum::routing::post(download::route))
}
