mod completion;
mod error;
#[allow(dead_code)]
mod raw;
mod stream;
#[cfg(test)]
mod test;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub use completion::{File, Message, MessageToolCall, MessageToolResult, Model, ModelBuilder, Openrouter, Tool};
pub use raw::{FinishReason, ResponseFormat};
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};
