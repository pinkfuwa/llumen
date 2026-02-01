//! Llumen Backend - LLM Chat Application Server
//!
//! This is the main entry point for the Llumen backend service. The backend is built with:
//! - Axum: web framework for handling HTTP requests
//! - SeaORM: database ORM for SQLite
//! - Tokio: async runtime
//! - OpenRouter: LLM API integration
//!
//! The server exposes REST APIs for chat management, user authentication, message handling,
//! and model discovery. It also serves the compiled frontend as static files.
//!
//! # Architecture Overview
//!
//! ## Core Components
//!
//! 1. **AppState**: Global application state containing database connection, encryption keys,
//!    and the main chat processing pipeline (Context).
//!
//! 2. **Chat Pipeline (chat/context.rs)**: Manages LLM completion requests, handles streaming
//!    responses, and coordinates between multiple chat modes (normal, search, deep research).
//!
//! 3. **API Routes**: Organized into modules for chat, user, message, model, file, and auth
//!    operations. Each route validates requests and interacts with the database and LLM API.
//!
//! 4. **Middlewares**: Handles authentication (PASETO tokens), compression (Zstd), and logging.
//!
//! 5. **OpenRouter Client**: Abstracts interactions with OpenRouter's LLM API, including
//!    streaming completions and tool calling.
//!
//! The MiMalloc allocator is used for better performance on memory-constrained systems.

#![deny(unsafe_code)]

mod chat;
mod config;
mod errors;
mod middlewares;
mod openrouter;
mod routes;
// TODO: rewrite runner(along with repl tool)
#[allow(dead_code)]
mod runner;
mod utils;

use std::{path::PathBuf, sync::Arc};

use anyhow::Context as _;
use axum::{Router, middleware};
use chat::Context;
use dotenv::var;
use entity::prelude::*;

use migration::MigratorTrait;
use mimalloc::MiMalloc;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{ConnectionTrait, Database, DbConn, EntityTrait};
use tokio::{net::TcpListener, signal};
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
///
/// It's wrapped in Arc<AppState> and passed through Axum middleware to all route handlers.
/// This enables stateful operations across async request handling.
///
/// # Fields
///
/// * `conn`: Database connection pool for all database operations via SeaORM
/// * `key`: Encryption key for PASETO token generation/validation used in authentication
/// * `hasher`: Password hasher for user authentication and security
/// * `processor`: Main chat processing pipeline context managing LLM completions
/// * `blob`: Blob database for storing binary data and file uploads
/// * `user_header`: Optional HTTP header name for header-based authentication (SSO/proxy integration)
pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub hasher: Hasher,
    pub processor: Arc<Context>,
    pub blob: Arc<BlobDB>,
    pub auth_header: Option<String>,
}

/// Attempts to load the OpenRouter API key from environment variables.
///
/// The API key is required to make requests to the OpenRouter API for LLM completions.
/// If not found, prints instructions for obtaining a key and exits gracefully.
fn load_api_key() -> String {
    match (var("API_KEY"), var("OPENA_API_KEY")) {
        (Ok(key), _) => key,
        (_, Ok(key)) => key,
        _ => {
            println!("Error: API_KEY environment variable not found.");
            println!("Note: llumen read environment variable as well as .env file.");
            println!("You can get a key from https://openrouter.ai/keys");
            println!("Or use alternative setup:");
            println!(
                "- configuration: https://github.com/pinkfuwa/llumen/blob/main/docs/user/configuration.md"
            );
            println!(
                "- documentation: https://llumen-docs.easonabc.eu.org/user-guide/model-config"
            );

            #[cfg(windows)]
            {
                use std::io::{self, Read};
                println!("Press Enter to exit...");
                io::stdin().read_exact(&mut [0u8]).unwrap();
            }
            std::process::exit(1);
        }
    }
}

/// Handles graceful shutdown signals.
///
/// Listens for either Ctrl+C (SIGINT) on all platforms or SIGTERM on Unix systems.
/// This allows the server to clean up resources and finish in-flight requests before stopping.
async fn shutdown_signal() {
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
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    crate::utils::logger::init();

    #[cfg(feature = "tracing")]
    let _main_span = info_span!("llumen_backend_startup").entered();

    let api_key = load_api_key();
    let api_base = var("API_BASE").unwrap_or_else(|_| {
        var("OPENAI_API_BASE").unwrap_or("https://openrouter.ai/api".to_string())
    });
    let data_path = PathBuf::from(var("DATA_PATH").unwrap_or(".".to_owned()));
    let mut database_path = data_path.clone();
    database_path.push("db.sqlite");
    let database_url = format!(
        "sqlite://{}?mode=rwc",
        database_path.display().to_string().replace('\\', "/")
    );
    let mut blob_path = data_path.clone();
    blob_path.push("blobs.redb");
    let bind_addr = var("BIND_ADDR").unwrap_or("0.0.0.0:8001".to_owned());

    #[cfg(feature = "tracing")]
    let _db_span = info_span!("database_initialization").entered();

    migration::migrate(&database_url)
        .await
        .expect("Migration failed");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");

    migration::Migrator::up(&conn, None)
        .await
        .expect("Cannot migrate database");

    conn.execute(sea_orm::Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA journal_mode = WAL;PRAGMA synchronous = normal;PRAGMA busy_timeout=1000;"
            .to_owned(),
    ))
    .await
    .expect("Failed to set pragmas");

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

    let openrouter = openrouter::Openrouter::new(api_key, api_base);

    let blob = Arc::new(
        BlobDB::new_from_path(blob_path)
            .await
            .expect("Cannot open blob db"),
    );

    let processor = Arc::new(
        Context::new(conn.clone(), openrouter, blob.clone())
            .expect("Failed to create pipeline context"),
    );

    let auth_header = var("TRUSTED_HEADER").ok();

    utils::file_cleanup::FileCleanupService::new(conn.clone(), blob.clone()).start();

    let state = Arc::new(AppState {
        conn,
        key,
        hasher: Hasher::default(),
        processor,
        blob,
        auth_header,
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
                // only compress plain text content
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

    log::info!("Listening on http://{}", bind_addr);

    #[cfg(feature = "tracing")]
    let _server_span = info_span!("server_startup", bind_addr = %bind_addr).entered();

    let tcp = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
