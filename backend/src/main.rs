mod config;
mod errors;
mod middlewares;
mod openrouter;
mod routes;
mod utils;

use std::sync::Arc;

use anyhow::Context;
use axum::{Router, middleware};
use dotenv::var;
use entity::prelude::*;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{Database, DbConn, EntityTrait};
use tokio::net::TcpListener;
use utils::password_hash::Hasher;
use utils::sse::{SseContext, spawn_sse};

#[cfg(feature = "dev")]
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub sse: SseContext,
    pub api_key: String,
    pub hasher: Hasher,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = var("DATABASE_URL").unwrap_or("sqlite://db.sqlite?mode=rwc".to_owned());
    let bind_addr = var("BIND_ADDR").unwrap_or("0.0.0.0:8001".to_owned());
    let api_key = var("API_KEY").expect("API_KEY is required");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");

    let key = SymmetricKey::from(
        &Config::find_by_id("paseto_key")
            .one(&conn)
            .await
            .unwrap()
            .context("Cannot find paseto key")
            .unwrap()
            .value,
    )
    .expect("Cannot parse paseto key");

    let sse = spawn_sse(conn.clone());
    let state = Arc::new(AppState {
        conn,
        key,
        sse,
        api_key,
        hasher: Hasher::default(),
    });

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/chat", routes::chat::routes())
                .nest("/user", routes::user::routes())
                .nest("/message", routes::message::routes())
                .nest("/model", routes::model::routes())
                .layer(middleware::from_extractor_with_state::<
                    middlewares::auth::Middleware,
                    _,
                >(state.clone()))
                .nest("/auth", routes::auth::routes()),
        )
        .with_state(state);

    #[cfg(feature = "dev")]
    let app = app.layer(
        CorsLayer::new()
            .allow_methods(AllowMethods::any())
            .allow_origin(AllowOrigin::any())
            .allow_headers(AllowHeaders::list([
                http::header::AUTHORIZATION,
                http::header::CONTENT_TYPE,
            ])),
    );

    let tcp = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(tcp, app).await.unwrap();
}
