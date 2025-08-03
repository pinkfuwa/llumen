mod create;
mod delete;
mod read;
mod write;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::route))
        .route("/delete", post(delete::route))
        .route("/read", post(read::route))
        .route("/write", post(write::route))
}
