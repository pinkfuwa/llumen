mod completion;
mod error;
#[allow(dead_code)]
mod raw;
mod stream;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub use completion::{File, Message, MessageToolCall, MessageToolResult, Model, Openrouter, Tool};
pub use raw::FinishReason;
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};
