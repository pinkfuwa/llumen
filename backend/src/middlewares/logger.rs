use std::{
    pin::Pin,
    task::{Context, Poll, ready},
};

use futures_util::Future;
use http::{Method, Request, Response};
use tower::{Layer, Service};

pub struct ResponseFuture<F> {
    response_future: F,
    uri: http::Uri,
    method: http::Method,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoggerLayer;

impl<S> Layer<S> for LoggerLayer {
    type Service = Logger<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Logger { inner }
    }
}

#[derive(Debug, Clone)]
pub struct Logger<S> {
    inner: S,
}

impl<'a, S, T, U> Service<Request<T>> for Logger<S>
where
    S: Service<Request<T>, Response = Response<U>>,
    S::Future: Unpin,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<T>) -> Self::Future {
        let uri = req.uri().clone();
        let method = req.method().clone();
        let response_future = self.inner.call(req);

        ResponseFuture {
            response_future,
            uri,
            method,
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
        let response: Response<B> = ready!(pin_response_future.poll(cx))?;

        if matches!(self.method, Method::POST) {
            log::info!(
                "serving api {} {}",
                self.uri.path(),
                response.status().as_u16()
            );
        }

        Poll::Ready(Ok(response))
    }
}
