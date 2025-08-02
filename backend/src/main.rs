mod errors;
mod middlewares;
mod openrouter;
mod routes;
mod utils;

use std::sync::Arc;

use axum::{Router, middleware};
use dotenv::var;
use pasetors::{
    keys::{Generate, SymmetricKey},
    version4::V4,
};
use sea_orm::{Database, DbConn};
use tokio::net::TcpListener;
use utils::sse::{SseContext, spawn_sse};

pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub sse: SseContext,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let database_url = var("DATABASE_URL").unwrap_or("sqlite://db.sqlite?mode=rwc".to_owned());
    let bind_addr = var("BIND_ADDR").unwrap_or("0.0.0.0:8001".to_owned());
    let api_key = var("API_KEY").expect("API_KEY is required");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");
    let sse = spawn_sse(conn.clone());
    let state = Arc::new(AppState {
        conn,
        key: SymmetricKey::generate().expect("Cannot generate key"),
        sse,
    });

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/chat", routes::chat::routes())
                .nest("/user", routes::user::routes())
                // .nest("/message",routes::message)
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    middlewares::auth::middleware,
                ))
                .nest("/auth", routes::auth::routes()),
        )
        .with_state(state);

    let tcp = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(tcp, app).await.unwrap();
}
