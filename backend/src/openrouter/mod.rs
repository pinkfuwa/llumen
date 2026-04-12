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

pub static HTTP_REFERER: &str = "https://pinkfuwa.github.io/llumen/";
pub static X_TITLE: &str = "llumen";

pub use annotation::extract_url_citations;
pub use chat::{ChatCompletion, StructuredCompletion};
pub use error::Error;
pub use image_gen::{AspectRatio, ImageGenOutput};
pub use message::{File, GeneratedImage, Message, MessageToolCall, MessageToolResult};
pub use model::{Capability, MaybeCapability, Model, ModelBuilder};
pub use openrouter::Openrouter;
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use raw::{FinishReason, Image};
pub use stream::{
    StreamCompletion, StreamCompletionResp, StreamResult, StreamWithOrderedTokens, ToolCall,
};
