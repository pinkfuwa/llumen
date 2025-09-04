mod config;
mod errors;
mod middlewares;
mod openrouter;
mod prompts;
mod routes;
mod sse;
mod tools;
mod utils;

use std::sync::Arc;

use crate::{openrouter::Openrouter, prompts::PromptEnv, tools::ToolStore};
use anyhow::Context;
use axum::{Router, middleware};
use dotenv::var;
use entity::prelude::*;
use migration::MigratorTrait;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{Database, DbConn, EntityTrait};
use sse::SseContext;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use utils::password_hash::Hasher;

#[cfg(feature = "dev")]
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub sse: SseContext,
    pub prompt: PromptEnv,
    pub hasher: Hasher,
    pub openrouter: Openrouter,
    pub tools: ToolStore,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = var("DATABASE_URL").unwrap_or("sqlite://db.sqlite?mode=rwc".to_owned());
    let bind_addr = var("BIND_ADDR").unwrap_or("0.0.0.0:8001".to_owned());
    let static_dir = var("STATIC_DIR").unwrap_or("../frontend/build".to_owned());

    migration::migrate(&database_url)
        .await
        .expect("Migration failed");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");

    migration::Migrator::up(&conn, None)
        .await
        .expect("Cannot migrate database");

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

    let sse = SseContext::new(conn.clone());
    let prompt = PromptEnv::new(conn.clone());
    let openrouter = Openrouter::new();
    let tools = ToolStore::new();

    let state = Arc::new(AppState {
        conn,
        key,
        sse,
        hasher: Hasher::default(),
        openrouter,
        prompt,
        tools,
    });

    let var_name = Router::new();
    let app = var_name
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
        .fallback_service(
            ServiceBuilder::new().service(
                ServeDir::new(static_dir)
                    .precompressed_gzip()
                    .precompressed_br(),
            ),
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
