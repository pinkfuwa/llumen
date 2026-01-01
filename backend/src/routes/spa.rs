use axum::{
    body::Body,
    http::{Request, Response, StatusCode, Uri, header},
};
use http::HeaderValue;
use std::sync::OnceLock;

#[cfg(debug_assertions)]
use rust_embed_for_web::DynamicFile;
#[cfg(not(debug_assertions))]
use rust_embed_for_web::EmbeddedFile;

use rust_embed_for_web::{EmbedableFile, RustEmbed};

#[derive(RustEmbed)]
#[folder = "../frontend/build"]
#[gzip = false]
#[br = true]
pub struct SpaAssets;

static APP_VERSION: OnceLock<String> = OnceLock::new();

#[cfg(debug_assertions)]
fn build_body(file: &DynamicFile, br: &mut bool) -> Body {
    let mut data = file.data();

    if *br {
        if let Some(br_data) = file.data_br() {
            data = br_data;
        } else {
            *br = false;
        }
    }

    Body::from(data)
}

#[cfg(not(debug_assertions))]
fn build_body(file: &EmbeddedFile, br: &mut bool) -> Body {
    let mut data = file.data();

    if *br {
        if let Some(br_data) = file.data_br() {
            data = br_data;
        } else {
            *br = false;
        }
    }

    Body::from(data)
}

fn get_app_version() -> &'static str {
    fn load_version() -> Option<String> {
        let content = SpaAssets::get("_app/version.json")?;
        let data = content.data();
        let version_str = std::str::from_utf8(data.as_ref()).ok()?;

        let mut version = version_str
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        if let Some(strip_version) = version.strip_prefix("version") {
            version = strip_version.to_string();
        }
        Some(version)
    }
    fn random_version() -> String {
        let mut bytes = [0u8; 12];
        getrandom::fill(&mut bytes).unwrap();
        bytes
            .iter()
            .map(|&b| {
                const CHARSET: &[u8] =
                    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                CHARSET[(b as usize) % CHARSET.len()] as char
            })
            .collect::<String>()
    }
    APP_VERSION.get_or_init(|| load_version().unwrap_or_else(random_version))
}

fn extract_cookie_version(req: &Request<Body>) -> Option<String> {
    req.headers()
        .get(header::COOKIE)
        .and_then(|val| val.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .filter_map(|part| {
                    let part = part.trim();
                    part.find('=').and_then(|eq_pos| {
                        let name = part[..eq_pos].trim();
                        if name == "app_version" {
                            Some(part[eq_pos + 1..].trim().to_string())
                        } else {
                            None
                        }
                    })
                })
                .next()
        })
}

pub async fn spa_handler(uri: Uri, req: Request<Body>) -> Response<Body> {
    let mut br = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .map(|x| x.to_str().ok())
        .flatten()
        .map(|x| x.contains("br"))
        .unwrap_or_default();

    let current_version = extract_cookie_version(&req);
    let version = get_app_version();

    let file = SpaAssets::get(uri.path().trim_start_matches('/'))
        .unwrap_or(SpaAssets::get("index.html").unwrap());

    let body = build_body(&file, &mut br);

    let is_index = file.name().ends_with("index.html");

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, file.mime_type().unwrap_or_default());

    if current_version.map(|x| x == version).unwrap_or_default() {
        if !is_index {
            builder = builder.header(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=31536000, immutable"),
            );
        }
    } else {
        let cookie_val = format!("app_version={}; Path=/; SameSite=Lax", version);
        builder = builder
            .header(
                header::SET_COOKIE,
                HeaderValue::from_str(&cookie_val).unwrap(),
            )
            .header(
                header::HeaderName::from_static("clear-site-data"),
                HeaderValue::from_static("\"cache\""),
            );
    }

    if br {
        builder = builder.header(header::CONTENT_ENCODING, HeaderValue::from_static("br"));
    }

    builder.body(body).unwrap()
}
