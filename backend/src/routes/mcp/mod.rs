mod check;
mod create;
mod delete;
mod list;
mod write;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/check", post(check::route))
        .route("/create", post(create::route))
        .route("/delete", post(delete::route))
        .route("/list", post(list::route))
        .route("/write", post(write::route))
}
