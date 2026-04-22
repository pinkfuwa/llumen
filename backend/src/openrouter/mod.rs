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

pub static LLUMEN_URL: &str = "https://pinkfuwa.github.io/llumen/";
pub static LLUMEN_NAME: &str = "llumen";
pub const HTTP_REFERER: &str = "HTTP-Referer";
pub const X_TITLE: &str = "X-OpenRouter-Title";

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
