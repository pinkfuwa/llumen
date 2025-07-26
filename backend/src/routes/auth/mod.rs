use std::sync::Arc;

use axum::{Router, routing::post};

use crate::AppState;

mod login;
mod renew;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login::route))
        .route("/renew", post(renew::route))
}
