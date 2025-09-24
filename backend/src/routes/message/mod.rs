mod create;
mod delete;
mod paginate;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::route))
        .route("/delete", post(delete::route))
        .route("/paginate", post(paginate::route))
}
