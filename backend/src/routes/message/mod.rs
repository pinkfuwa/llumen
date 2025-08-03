mod create;
mod paginate;
mod write;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::route))
        .route("/write", post(write::route))
        .route("/paginate", post(paginate::route))
}
