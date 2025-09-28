mod chat;
mod config;
mod errors;
mod middlewares;
mod openrouter;
mod routes;
mod utils;

use std::sync::Arc;

use anyhow::Context as _;
use axum::{Router, middleware};
use chat::Context;
use dotenv::var;
use entity::prelude::*;
use middlewares::cache_control::CacheControlLayer;
use migration::MigratorTrait;
use pasetors::{keys::SymmetricKey, version4::V4};
use sea_orm::{Database, DbConn, EntityTrait};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use utils::{blob::BlobDB, password_hash::Hasher};

#[cfg(feature = "dev")]
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

pub struct AppState {
    pub conn: DbConn,
    pub key: SymmetricKey<V4>,
    pub hasher: Hasher,
    pub processor: Arc<Context>,
    pub blob: Arc<BlobDB>,
}

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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter::Targets::new().with_target("backend", Level::TRACE))
        .init();

    let api_key = load_api_key();
    let api_base = var("API_BASE").unwrap_or("https://openrouter.ai/".to_string());
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

    let var_name = Router::new();
    let app = var_name
        .nest(
            "/api",
            Router::new()
                .nest("/chat", routes::chat::routes())
                .nest("/user", routes::user::routes())
                .nest("/message", routes::message::routes())
                .nest("/model", routes::model::routes())
                .nest("/file", routes::file::routes())
                .layer(middleware::from_extractor_with_state::<
                    middlewares::auth::Middleware,
                    _,
                >(state.clone()))
                .nest("/auth", routes::auth::routes()),
        )
        .fallback_service(
            ServiceBuilder::new().layer(CacheControlLayer).service(
                ServeDir::new(static_dir.to_owned())
                    .precompressed_gzip()
                    .precompressed_br()
                    .fallback(
                        ServeFile::new(format!("{}/index.html", static_dir))
                            .precompressed_br()
                            .precompressed_gzip(),
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

    tracing::info!("Listening on http://{}", bind_addr);

    let tcp = TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(tcp, app).await.unwrap();
}
