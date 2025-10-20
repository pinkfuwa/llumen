use std::{
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};

use http::header::HeaderName;
use http::{
    HeaderValue, Request, Response,
    header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, SET_COOKIE},
};
use tower::{Layer, Service};

pub struct ResponseFuture<F> {
    response_future: F,
    version: Arc<String>,
    old_version: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CacheControlLayer {
    version: Arc<String>,
}

impl CacheControlLayer {
    pub fn new() -> Self {
        let version = (0..12)
            .map(|_| fastrand::alphanumeric())
            .collect::<String>();
        Self {
            version: Arc::new(version),
        }
    }
    pub async fn try_load_version(
        &mut self,
        svk_dir: impl AsRef<Path>,
    ) -> Result<(), std::io::Error> {
        let mut path = svk_dir.as_ref().to_path_buf();
        path.push("_app");
        path.push("version.json");

        let content = tokio::fs::read_to_string(&path).await?;
        let mut version = content
            .trim()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        if let Some(strip_version) = version.strip_prefix("version") {
            version = strip_version.to_string();
        }

        self.version = Arc::new(version);

        Ok(())
    }
}

impl<S> Layer<S> for CacheControlLayer {
    type Service = CacheControl<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheControl {
            inner,
            version: self.version.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheControl<S> {
    inner: S,
    version: Arc<String>,
}

impl<'a, S, T, B> Service<Request<T>> for CacheControl<S>
where
    S: Service<Request<T>, Response = Response<B>>,
    S::Future: Unpin,
{
    type Response = Response<B>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let old_version = req
            .headers()
            .get(COOKIE)
            .and_then(|val| val.to_str().ok())
            .and_then(|cookie_str| {
                cookie_str
                    .split(';')
                    .filter_map(|part| {
                        let part = part.trim();
                        part.find('=').and_then(|eq_pos| {
                            let name = part[..eq_pos].trim();
                            if name == "app_version" {
                                let value = part[eq_pos + 1..].trim().to_string();
                                Some(value)
                            } else {
                                None
                            }
                        })
                    })
                    .next()
            });

        let version = self.version.clone();
        let response_future = self.inner.call(req);

        ResponseFuture {
            response_future,
            version,
            old_version,
        }
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>> + Unpin,
{
    type Output = Result<Response<B>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin_response_future = Pin::new(&mut self.response_future);
        let mut response: Response<B> = ready!(pin_response_future.poll(cx))?;

        let headers = response.headers_mut();

        headers.insert(
            HeaderName::from_static("x-app-version"),
            HeaderValue::from_str(&self.version).unwrap(),
        );

        let mut need_cookie = false;
        if let Some(ref old) = self.old_version {
            if old.as_str() != self.version.as_str() {
                headers.insert(
                    HeaderName::from_static("clear-site-data"),
                    HeaderValue::from_static("\"cache\""),
                );
                need_cookie = true;
            }
        }

        let mut is_html = false;
        if let Some(content_type) = headers.get(CONTENT_TYPE) {
            if let Ok(ct_str) = content_type.to_str() {
                if ct_str.starts_with("text/html") {
                    is_html = true;
                }
            }
        }

        if is_html || need_cookie {
            let cookie_val = format!("app_version={}; Path=/; SameSite=Lax", self.version);
            headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());
        }

        if !is_html {
            headers.insert(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
        }

        Poll::Ready(Ok(response))
    }
}
