use axum::{
    body::Body,
    http::{Request, Response, StatusCode, Uri, header},
};
use http::HeaderValue;

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

pub async fn spa_handler(uri: Uri, req: Request<Body>) -> Response<Body> {
    let mut br = req
        .headers()
        .get(header::ACCEPT_ENCODING)
        .map(|x| x.to_str().ok())
        .flatten()
        .map(|x| x.contains("br"))
        .unwrap_or_default();

    let file = SpaAssets::get(uri.path().trim_start_matches('/'))
        .unwrap_or(SpaAssets::get("index.html").unwrap());

    let body = build_body(&file, &mut br);

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, file.mime_type().unwrap_or_default());

    if br {
        builder = builder.header(header::CONTENT_ENCODING, HeaderValue::from_static("br"));
    }

    builder.body(body).unwrap()
}
