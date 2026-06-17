use std::sync::OnceLock;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::config;

struct RateLimitState {
    count: u64,
    window_start: Instant,
}

static STATE: OnceLock<Mutex<RateLimitState>> = OnceLock::new();

pub async fn handle(request: Request<Body>, next: Next) -> Response {
    let state = STATE.get_or_init(|| {
        Mutex::new(RateLimitState {
            count: 0,
            window_start: Instant::now(),
        })
    });

    let mut state = state.lock().await;
    let now = Instant::now();

    if now.duration_since(state.window_start).as_secs() >= config::LOGIN_RATE_LIMIT_WINDOW_SECS {
        state.count = 0;
        state.window_start = now;
    }

    state.count += 1;

    if state.count > config::LOGIN_RATE_LIMIT_MAX {
        log::warn!("rate limit exceeded for /auth/login");
        return (StatusCode::TOO_MANY_REQUESTS, "Too many requests").into_response();
    }

    drop(state);
    next.run(request).await
}
