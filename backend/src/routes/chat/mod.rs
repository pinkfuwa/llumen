mod create;
mod delete;
mod halt;
mod paginate;
mod read;
mod sse;
mod write;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/sse", post(sse::route))
        .route("/delete", post(delete::route))
        .route("/paginate", post(paginate::route))
        .route("/read", post(read::route))
        .route("/create", post(create::route))
        .route("/halt", post(halt::route))
        .route("/write", post(write::route))
}
