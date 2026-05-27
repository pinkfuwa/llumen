use std::sync::LazyLock;

use reqwest::header::{HeaderMap, HeaderValue};

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
