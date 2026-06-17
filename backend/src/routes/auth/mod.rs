use std::sync::Arc;

use axum::{Router, middleware, routing::post};

use crate::AppState;

mod header_auth;
mod helper;
mod login;
mod renew;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/login",
            post(login::route).layer(middleware::from_fn(crate::middlewares::rate_limit::handle)),
        )
        .route("/renew", post(renew::route))
        .route("/header", post(header_auth::route))
}
