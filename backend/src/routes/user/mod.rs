use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

mod create;
mod delete;
mod list;
mod read;
mod update;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::route))
        .route("/delete", post(delete::route))
        .route("/read", post(read::route))
        .route("/update", post(update::route))
        .route("/list", post(list::route))
}
