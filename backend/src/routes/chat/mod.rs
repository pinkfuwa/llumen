use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
}
