mod chat;
mod config;
mod errors;
mod middlewares;
mod openrouter;
mod routes;
mod runner;
mod utils;

use std::sync::Arc;

use anyhow::Context as _;
use axum::{Router, middleware};
use chat::Context;
use dotenv::var;
use entity::prelude::*;
use middlewares::cache_control::CacheControlLayer;
use migration::MigratorTrait;
use mimalloc::MiMalloc;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{ConnectionTrait, Database, DbConn, EntityTrait};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use utils::{blob::BlobDB, password_hash::Hasher};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(feature = "dev")]
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

// TODO: musl allocator sucks, but compile time is important, too.

pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub hasher: Hasher,
    pub processor: Arc<Context>,
    pub blob: Arc<BlobDB>,
}

// immediately exit confuse user
fn load_api_key() -> String {
    match var("API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Warning: API_KEY environment variable not found.");
            println!("You can get a key from https://openrouter.ai/keys");
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

    let api_key = load_api_key();
    let api_base = var("OPENAI_API_BASE").unwrap_or("https://openrouter.ai/api".to_string());
    let database_url = var("DATABASE_URL").unwrap_or("sqlite://db.sqlite?mode=rwc".to_owned());
    let bind_addr = var("BIND_ADDR").unwrap_or("0.0.0.0:8001".to_owned());
    let static_dir = var("STATIC_DIR").unwrap_or(
        option_env!("STATIC_DIR")
            .unwrap_or("../frontend/build")
            .to_owned(),
    );
    let blob_url = var("BLOB_URL").unwrap_or("./blobs.redb".to_owned());

    migration::migrate(&database_url)
        .await
        .expect("Migration failed");

    let conn = Database::connect(database_url)
        .await
        .expect("Cannot connect to database");

    migration::Migrator::up(&conn, None)
        .await
        .expect("Cannot migrate database");

    // Some side note about memory:
    // llumen is design to run on 1GB memory
    // 1. sqlite page cache: 128MB
    // 2. backend thread: 4MB * 4 = 16MB
    // 3. heap memory: 256MB
    // 4. lua runtime: 64MB * 8 = 512MB
    conn.execute(sea_orm::Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA journal_mode = WAL;PRAGMA synchronous = normal;".to_owned(),
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
        BlobDB::new_from_path(blob_url)
            .await
            .expect("Cannot open blob db"),
    );

    let processor = Arc::new(
        Context::new(conn.clone(), openrouter, blob.clone())
            .expect("Failed to create pipeline context"),
    );

    let state = Arc::new(AppState {
        conn,
        key,
        hasher: Hasher::default(),
        processor,
        blob,
    });

    let mut cache_control = CacheControlLayer::new();

    if let Err(err) = cache_control.try_load_version(&static_dir).await {
        log::warn!("Fail to load svelte kit's build version. {}", err);
    }

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
        .fallback_service(
            // side notes about artifact size:
            // 1. br sized about 1.3Mb, uncompressed sized about 4Mb
            // 2. Rust binary sized about 6Mb
            ServiceBuilder::new().layer(cache_control).service(
                ServeDir::new(static_dir.to_owned())
                    .precompressed_br()
                    .fallback(
                        ServeFile::new(format!("{}/index.html", static_dir)).precompressed_br(),
                    ),
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

    log::info!("Listening on http://{}", bind_addr);

    let tcp = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
