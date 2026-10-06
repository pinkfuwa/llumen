use std::{borrow::Cow, io::Read};

use axum::{
    body::Body,
    http::{HeaderMap, HeaderValue, Method, Response, StatusCode, header},
};
use compact_embed::{Asset, Encoding};
use tokio::sync::{Semaphore, mpsc};
use tokio_stream::wrappers::ReceiverStream;

// Each decoder also owns a Zstd window. Limit decoder state as well as queued
// output.
static DECODERS: Semaphore = Semaphore::const_new(2);

fn quality(value: &str) -> Option<u16> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if fraction.len() > 3 || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    match whole {
        "1" if fraction.bytes().all(|byte| byte == b'0') => Some(1000),
        "0" => {
            let mut weight = 0;
            for (index, byte) in fraction.bytes().enumerate() {
                weight += u16::from(byte - b'0') * [100, 10, 1][index];
            }
            Some(weight)
        }
        _ => None,
    }
}

fn select_encoding(headers: &HeaderMap, stored: Encoding) -> Option<Encoding> {
    let mut zstd: Option<u16> = None;
    let mut identity: Option<u16> = None;
    let mut wildcard: Option<u16> = None;
    for value in headers.get_all(header::ACCEPT_ENCODING) {
        let Ok(value) = value.to_str() else { continue };
        for item in value.split(',') {
            let mut parts = item.split(';');
            let token = parts.next().unwrap_or_default().trim();
            let mut weight = 1000;
            let mut has_quality = false;
            for parameter in parts {
                let Some((name, value)) = parameter.trim().split_once('=') else {
                    weight = 0;
                    break;
                };
                if !name.trim().eq_ignore_ascii_case("q") || has_quality {
                    weight = 0;
                    break;
                }
                weight = quality(value.trim()).unwrap_or(0);
                has_quality = true;
            }
            let selected = if token.eq_ignore_ascii_case("zstd") {
                &mut zstd
            } else if token.eq_ignore_ascii_case("identity") {
                &mut identity
            } else if token == "*" {
                &mut wildcard
            } else {
                continue;
            };
            *selected = Some(selected.map_or(weight, |previous| previous.min(weight)));
        }
    }
    let zstd = zstd.or(wildcard).unwrap_or(0);
    let identity = identity.unwrap_or(if wildcard == Some(0) { 0 } else { 1000 });
    if stored == Encoding::Zstd && zstd > 0 && zstd >= identity {
        Some(Encoding::Zstd)
    } else if identity > 0 {
        Some(Encoding::Identity)
    } else {
        None
    }
}

fn decode_chunks(
    asset: &Asset,
    sender: &mpsc::Sender<Result<Vec<u8>, std::io::Error>>,
) -> std::io::Result<()> {
    if sender.is_closed() {
        return Ok(());
    }
    let mut reader = asset.reader()?;
    let mut total = 0u64;
    loop {
        if sender.is_closed() {
            return Ok(());
        }
        let mut chunk = vec![0; 8192];
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            if total != asset.original_size {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "embedded asset decoded length mismatch",
                ));
            }
            return Ok(());
        }
        total = total.saturating_add(read as u64);
        if total > asset.original_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "embedded asset exceeded decoded length",
            ));
        }
        chunk.truncate(read);
        if sender.blocking_send(Ok(chunk)).is_err() {
            return Ok(());
        }
    }
}

async fn decoded_body(asset: Asset) -> Result<Body, tokio::sync::AcquireError> {
    let permit = DECODERS.acquire().await?;
    let (sender, receiver) = mpsc::channel::<Result<Vec<u8>, std::io::Error>>(2);
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            if let Err(error) = decode_chunks(&asset, &sender) {
                log::error!("Unable to decode frontend asset: {error}");
                if sender.blocking_send(Err(error)).is_err() {
                    log::debug!("Frontend asset receiver disconnected before the decoder error could be delivered");
                }
            }
        })
        .await;
        if let Err(error) = result {
            log::error!("Frontend asset decoder task failed: {error}");
        }
    });
    Ok(Body::from_stream(ReceiverStream::new(receiver)))
}

pub async fn response(asset: Asset, headers: &HeaderMap, method: &Method) -> Response<Body> {
    let Some(encoding) = select_encoding(headers, asset.encoding) else {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::NOT_ACCEPTABLE;
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        return response;
    };
    let length = if encoding == Encoding::Zstd {
        asset.data.len() as u64
    } else {
        asset.original_size
    };
    let mime_type = asset.mime_type;
    let body = if method == Method::HEAD {
        Body::empty()
    } else if encoding == Encoding::Identity && asset.encoding == Encoding::Zstd {
        match decoded_body(asset).await {
            Ok(body) => body,
            Err(error) => {
                log::error!("Unable to acquire frontend decoder permit: {error}");
                let mut response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return response;
            }
        }
    } else {
        match asset.data {
            Cow::Borrowed(bytes) => Body::from(bytes),
            Cow::Owned(bytes) => Body::from(bytes),
        }
    };
    let mut response = Response::new(body);
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime_type));
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from(length));
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    if encoding == Encoding::Zstd {
        headers.insert(header::CONTENT_ENCODING, HeaderValue::from_static("zstd"));
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use compact_embed::Embed;

    #[derive(Embed)]
    #[embed(folder = "tests/assets", dynamic = false)]
    struct Fixtures;

    fn headers(encoding: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, encoding.parse().unwrap());
        headers
    }

    #[test]
    fn negotiates_tokens_quality_wildcards_and_identity() {
        for (value, expected) in [
            ("", Some(Encoding::Identity)),
            ("gzip, zstd", Some(Encoding::Zstd)),
            ("ZSTD", Some(Encoding::Zstd)),
            ("xzstd", Some(Encoding::Identity)),
            ("zstd;q=0", Some(Encoding::Identity)),
            ("zstd;q=0.5, identity;q=0.8", Some(Encoding::Identity)),
            ("zstd;q=0.8, identity;q=0.5", Some(Encoding::Zstd)),
            ("*", Some(Encoding::Zstd)),
            ("*;q=0, zstd;q=1", Some(Encoding::Zstd)),
            ("*;q=1, zstd;q=0", Some(Encoding::Identity)),
            ("zstd;q=0, identity;q=0", None),
            ("*;q=0", None),
            ("zstd;q=invalid", Some(Encoding::Identity)),
            ("zstd;q", Some(Encoding::Identity)),
            ("zstd;q=1.001", Some(Encoding::Identity)),
            ("zstd;q=1;q=0", Some(Encoding::Identity)),
        ] {
            assert_eq!(
                select_encoding(&headers(value), Encoding::Zstd),
                expected,
                "{value}"
            );
        }
        assert_eq!(
            select_encoding(&HeaderMap::new(), Encoding::Zstd),
            Some(Encoding::Identity)
        );
        assert_eq!(
            select_encoding(&headers("zstd"), Encoding::Identity),
            Some(Encoding::Identity)
        );
        assert_eq!(
            select_encoding(&headers("zstd, identity;q=0"), Encoding::Identity),
            None
        );
        let mut headers = headers("gzip");
        headers.append(header::ACCEPT_ENCODING, HeaderValue::from_static("zstd"));
        assert_eq!(
            select_encoding(&headers, Encoding::Zstd),
            Some(Encoding::Zstd)
        );
        headers.append(
            header::ACCEPT_ENCODING,
            HeaderValue::from_static("zstd;q=0"),
        );
        assert_eq!(
            select_encoding(&headers, Encoding::Zstd),
            Some(Encoding::Identity)
        );
    }

    #[tokio::test]
    async fn zstd_is_delivered_directly_with_original_mime() {
        let asset = Fixtures::get("code.js").unwrap().unwrap();
        assert_eq!(asset.encoding, Encoding::Zstd);
        let stored = asset.data.clone();
        let response = response(asset, &headers("zstd"), &Method::GET).await;
        assert_eq!(response.headers()[header::CONTENT_ENCODING], "zstd");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/javascript; charset=utf-8"
        );
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            stored.len().to_string()
        );
        assert_eq!(response.headers()[header::VARY], "Accept-Encoding");
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .as_ref(),
            stored.as_ref()
        );
    }

    #[tokio::test]
    async fn identity_is_stream_decoded_byte_for_byte() {
        let asset = Fixtures::get("code.js").unwrap().unwrap();
        let response = response(asset, &headers("zstd;q=0"), &Method::GET).await;
        assert!(!response.headers().contains_key(header::CONTENT_ENCODING));
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            include_bytes!("../../tests/assets/code.js")
                .len()
                .to_string()
        );
        assert_eq!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .as_ref(),
            include_bytes!("../../tests/assets/code.js")
        );
    }

    #[tokio::test]
    async fn head_has_correct_headers_without_decoding() {
        let mut asset = Fixtures::get("code.js").unwrap().unwrap();
        asset.data = Cow::Borrowed(b"invalid Zstd");
        let length = asset.original_size;
        let response = response(asset, &headers("identity"), &Method::HEAD).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            length.to_string()
        );
        assert!(
            to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn raw_assets_remain_raw_and_impossible_negotiation_is_406() {
        let asset = Fixtures::get("tiny.txt").unwrap().unwrap();
        let plain = response(asset.clone(), &headers("zstd"), &Method::GET).await;
        assert!(!plain.headers().contains_key(header::CONTENT_ENCODING));
        assert_eq!(
            to_bytes(plain.into_body(), usize::MAX)
                .await
                .unwrap()
                .as_ref(),
            b"x"
        );
        let unavailable = response(asset, &headers("zstd, identity;q=0"), &Method::GET).await;
        assert_eq!(unavailable.status(), StatusCode::NOT_ACCEPTABLE);
        assert_eq!(unavailable.headers()[header::VARY], "Accept-Encoding");
    }

    #[tokio::test]
    async fn broken_compressed_data_produces_a_body_error() {
        let mut asset = Fixtures::get("code.js").unwrap().unwrap();
        asset.data = Cow::Borrowed(b"broken");
        let response = response(asset, &HeaderMap::new(), &Method::GET).await;
        assert!(to_bytes(response.into_body(), usize::MAX).await.is_err());
    }

    #[tokio::test]
    async fn disconnected_stream_releases_decoder_capacity() {
        let asset = Asset {
            data: Cow::Borrowed(include_bytes!("../../tests/assets/large.txt.zst")),
            encoding: Encoding::Zstd,
            original_size: 65536,
            mime_type: "text/plain",
        };
        let first = decoded_body(asset.clone()).await.unwrap();
        let second = decoded_body(asset.clone()).await.unwrap();
        drop((first, second));
        let next = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let body = decoded_body(asset).await.unwrap();
            to_bytes(body, usize::MAX).await.unwrap()
        })
        .await
        .unwrap();
        assert_eq!(next.len(), 65536);
        assert!(next.iter().all(|byte| *byte == b'x'));
    }
}
