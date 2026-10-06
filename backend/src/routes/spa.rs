use axum::{
    body::Body,
    http::{Request, Response, StatusCode, Uri},
};
use compact_embed::Embed;

#[derive(Embed)]
#[embed(folder = "../frontend/build")]
struct SpaAssets;

pub async fn spa_handler(uri: Uri, req: Request<Body>) -> Response<Body> {
    match SpaAssets::get(uri.path().trim_start_matches('/')).and_then(|asset| match asset {
        Some(asset) => Ok(Some(asset)),
        None => SpaAssets::get("index.html"),
    }) {
        Ok(Some(asset)) => crate::utils::assets::response(asset, req.headers(), req.method()).await,
        Ok(None) => {
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::SERVICE_UNAVAILABLE;
            response
        }
        Err(error) => {
            log::error!("Unable to load frontend asset: {error}");
            let mut response = Response::new(Body::empty());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_route_falls_back_to_html_or_returns_unavailable() {
        let index = SpaAssets::get("index.html").unwrap();
        let request = Request::builder()
            .method("HEAD")
            .body(Body::empty())
            .unwrap();
        let response = spa_handler("/missing-frontend-route".parse().unwrap(), request).await;
        if let Some(index) = index {
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response.headers()[axum::http::header::CONTENT_TYPE],
                index.mime_type
            );
        } else {
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        }
        assert!(
            axum::body::to_bytes(response.into_body(), 0)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
