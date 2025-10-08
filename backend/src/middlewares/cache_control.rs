use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use http::{
    HeaderValue, Request, Response,
    header::{CACHE_CONTROL, CONTENT_TYPE},
};
use tower::{Layer, Service};

pub struct ResponseFuture<F> {
    response_future: F,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CacheControlLayer;

impl<S> Layer<S> for CacheControlLayer {
    type Service = CacheControl<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheControl { inner }
    }
}

#[derive(Debug, Clone)]
pub struct CacheControl<S> {
    inner: S,
}

impl<'a, S, T, U> Service<Request<T>> for CacheControl<S>
where
    S: Service<Request<T>, Response = Response<U>>,
    U: Default,
    S::Future: Unpin,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let response_future = self.inner.call(req);

        ResponseFuture { response_future }
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>> + Unpin,
    B: Default,
{
    type Output = Result<Response<B>, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin_response_future = Pin::new(&mut self.response_future);
        let mut response: Response<B> = ready!(pin_response_future.poll(cx))?;

        if let Some(Ok(content_type)) = response.headers().get(CONTENT_TYPE).map(|x| x.to_str()) {
            if content_type != "text/html" {
                response
                    .headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static("max-age=604800"));
            }
        }

        Poll::Ready(Ok(response))
    }
}
