//! OpenRouter API client.
//!
//! Provides the [`Openrouter`] facade over chat, image, video, and embedding
//! endpoints. Model capabilities are lazily fetched and cached in
//! [`listing`] so the rest of the crate avoids repeated metadata calls.

use std::sync::LazyLock;

use futures_util::TryStreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use stream_json::IntoSerializer;

mod annotation;
mod chat;
mod error;
mod image_gen;
mod listing;
mod message;
mod model;
mod openrouter;
mod option;
#[allow(dead_code)]
mod raw;
mod stream;
mod video_gen;

/// Creates a request body from a streaming JSON serializer, dropping empty
/// chunks.
///
/// HTTP/2 treats a zero-length DATA frame as the end of the request body, so
/// the empty chunks that `stream_json` can emit truncate the request and make
/// the peer close the connection.
pub(super) fn json_body<T>(value: T) -> reqwest::Body
where
    T: IntoSerializer + Send + 'static,
    T::S: Send + 'static,
{
    reqwest::Body::wrap_stream(non_empty_chunks(value.into_stream()))
}

fn non_empty_chunks<S>(
    stream: S,
) -> impl futures_util::Stream<Item = Result<bytes::Bytes, stream_json::Error>> + Send
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, stream_json::Error>> + Send,
{
    stream.try_filter(|bytes| futures_util::future::ready(!bytes.is_empty()))
}

/// Static headers sent with every OpenRouter API request for attribution.
pub static OPENROUTER_HEADERS: LazyLock<HeaderMap> = LazyLock::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::HeaderName::from_static("http-referer"),
        HeaderValue::from_static("https://pinkfuwa.github.io/llumen/"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("x-openrouter-title"),
        HeaderValue::from_static("llumen"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("x-openrouter-categories"),
        HeaderValue::from_static("general-chat"),
    );
    headers
});

pub use chat::{ChatCompletion, StructuredCompletion};
pub use error::Error;
pub use image_gen::{AspectRatio, ImageGenOutput};
pub use message::{File, GeneratedImage, Message, MessageToolCall, MessageToolResult};
pub use model::{Capability, MaybeCapability, Model, ModelBuilder};
pub use openrouter::Openrouter;
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use stream::{
    StreamCompletion, StreamCompletionResp, StreamResult, StreamWithOrderedTokens, ToolCall,
};
pub use video_gen::{GeneratedVideo, VideoGenOutput, VideoGenerationOption};

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt;
    use stream_json::IntoSerializer;

    fn request() -> super::raw::CompletionReq {
        super::raw::CompletionReq {
            model: "azure/gpt-4o-mini".to_string(),
            messages: Vec::new(),
            stream: true,
            temperature: None,
            repeat_penalty: None,
            top_k: None,
            top_p: None,
            max_tokens: None,
            tools: Vec::new(),
            plugins: Vec::new(),
            usage: None,
            response_format: None,
            image_config: None,
            reasoning: Default::default(),
            modalities: Vec::new(),
            session_id: None,
        }
    }

    #[tokio::test]
    async fn json_body_drops_empty_chunks() {
        let raw_chunks = request()
            .into_stream()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert!(raw_chunks.iter().any(|chunk| chunk.is_empty()));

        let chunks = super::non_empty_chunks(request().into_stream())
            .try_collect::<Vec<_>>()
            .await
            .unwrap();

        assert!(!chunks.is_empty());
        assert!(chunks.iter().all(|chunk| !chunk.is_empty()));
        assert_eq!(
            raw_chunks.iter().map(|chunk| chunk.len()).sum::<usize>(),
            chunks.iter().map(|chunk| chunk.len()).sum::<usize>()
        );
    }
}
