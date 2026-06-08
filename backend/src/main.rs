//! Llumen Backend - LLM Chat Application Server
//!
//! This is the main entry point for the Llumen backend service. The backend is
//! built with:
//! - Axum: web framework for handling HTTP requests
//! - SeaORM: database ORM for SQLite
//! - Tokio: async runtime
//! - OpenRouter: LLM API integration
//!
//! The server exposes REST APIs for chat management, user authentication,
//! message handling, and model discovery. It also serves the compiled frontend
//! as static files.

#![deny(unsafe_code)]

mod chat;
mod config;
mod errors;
mod middlewares;
mod openrouter;
mod routes;
mod utils;

pub mod serde {
    pub use stream_json::serde::*;
}

pub mod error {
    pub use crate::openrouter::Error;
}

use std::sync::Arc;

use anyhow::Context as _;
use axum::{Router, middleware};
use chat::Context;
use config::{DB_BUSY_TIMEOUT_MS, DB_CACHE_SIZE};
use entity::prelude::*;

use migration::MigratorTrait;
use mimalloc::MiMalloc;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{ConnectionTrait, Database, DbConn, EntityTrait};
use tokio::{net::TcpListener, signal};
use utils::environment::Environment;
use utils::{blob::BlobDB, password_hash::Hasher};

#[cfg(feature = "tracing")]
use tracing::info_span;

/// Use MiMalloc allocator for better performance on memory-constrained systems
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(feature = "dev")]
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

// TODO: musl allocator sucks, but compile time is important, too.

/// AppState contains all shared server state accessible to request handlers.
pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub hasher: Hasher,
    pub chat: Arc<Context>,
    pub openrouter: Arc<crate::openrouter::Openrouter>,
    pub blob: Arc<BlobDB>,
    pub auth_header: Option<String>,
}

/// Handles graceful shutdown signals.
async fn shutdown_signal() {
    log::debug!("Shutdown signal handler started");
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    log::debug!("Shutdown signal received, shutting down");
}

fn setup_panic_handler() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
        std::process::exit(1);
    }));
}

#[tokio::main]
async fn main() {
    setup_panic_handler();

    dotenvy::dotenv().ok();

    #[cfg(feature = "cli")]
    let env = {
        let cli = utils::cli::CliArgs::parse_with_fallbacks();
        Environment::load_from(&cli)
    };

    #[cfg(not(feature = "cli"))]
    let env = Environment::load();

    crate::utils::logger::init(&env);

    #[cfg(feature = "tracing")]
    let _main_span = info_span!("llumen_backend_startup").entered();

    log::debug!("API key loaded, base: {}", env.api_base);
    log::debug!("Force OpenRouter mode: {}", env.force_openrouter);
    log::debug!("Data path: {}", env.data_path.display());
    let mut database_path = env.data_path.clone();
    database_path.push("db.sqlite");
    let database_url = format!(
        "sqlite://{}?mode=rwc",
        database_path.display().to_string().replace('\\', "/")
    );
    log::debug!("Database URL: {}", database_url);
    let mut blob_path = env.data_path.clone();
    blob_path.push("blobs.redb");
    log::debug!("Blob path: {}", blob_path.display());
    log::debug!("Bind address: {}", env.bind_addr);

    #[cfg(feature = "tracing")]
    let _db_span = info_span!("database_initialization").entered();

    migration::migrate(&database_url)
        .await
        .expect("Migration failed");
    log::debug!("Migration completed");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");
    log::debug!("Database connected");

    migration::Migrator::up(&conn, None)
        .await
        .expect("Cannot migrate database");
    log::debug!("Database migrated");

    conn.execute(sea_orm::Statement::from_string(
        conn.get_database_backend(),
        &format!(
            "PRAGMA journal_mode = WAL;PRAGMA synchronous = normal;PRAGMA busy_timeout={};PRAGMA cache_size = -{};",
            DB_BUSY_TIMEOUT_MS,
            DB_CACHE_SIZE/1024
        ),
    ))
    .await
    .expect("Failed to set pragmas");
    log::debug!("Pragmas set");

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
    log::debug!("Paseto key loaded");

    let openrouter = Arc::new(openrouter::Openrouter::new(
        env.api_key,
        env.api_base,
        env.force_openrouter,
    ));
    log::debug!("OpenRouter client created");

    let blob = Arc::new(
        BlobDB::new_from_path(blob_path)
            .await
            .expect("Cannot open blob db"),
    );
    log::debug!("Blob DB opened");

    let chat = Arc::new(
        Context::new(conn.clone(), openrouter.clone(), blob.clone())
            .expect("Failed to create pipeline context"),
    );
    log::debug!("Chat context created");

    utils::file_cleanup::FileCleanupService::new(conn.clone(), blob.clone()).start();

    let state = Arc::new(AppState {
        conn,
        key,
        hasher: Hasher::default(),
        chat,
        openrouter,
        blob,
        auth_header: env.auth_header,
    });

    #[cfg(feature = "tracing")]
    let _router_span = info_span!("router_setup").entered();

    let var_name = Router::new();
    let app = var_name
        .nest(
            "/api",
            Router::new()
                .nest("/chat", routes::chat::routes())
                .nest("/user", routes::user::routes())
                .nest("/message", routes::message::routes())
                .nest("/model", routes::model::routes())
                .layer(middlewares::compression::ZstdCompressionLayer)
                .nest("/file", routes::file::routes())
                .layer(middleware::from_extractor_with_state::<
                    middlewares::auth::Middleware,
                    _,
                >(state.clone()))
                .nest("/auth", routes::auth::routes())
                .layer(middlewares::logger::LoggerLayer),
        )
        .fallback(routes::spa::spa_handler)
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

    log::info!("Listening on http://{}", env.bind_addr);

    #[cfg(feature = "tracing")]
    let _server_span = info_span!("server_startup", bind_addr = %env.bind_addr).entered();

    let tcp = TcpListener::bind(env.bind_addr).await.unwrap();
    axum::serve(tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
