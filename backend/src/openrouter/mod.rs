mod error;
mod model;
mod openrouter;
#[allow(dead_code)]
mod raw;
mod stream;
#[cfg(test)]
mod test;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub use model::{Model, ModelBuilder};
pub use openrouter::{File, Message, MessageToolCall, MessageToolResult, Openrouter, Tool};
pub use raw::{FinishReason, ResponseFormat};
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};
