use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

mod create;
mod delete;
mod info;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::route))
        .route("/delete", post(delete::route))
        .route("/info", post(info::route))
}
