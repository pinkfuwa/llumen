mod error;
mod model;
mod openrouter;
mod option;
#[allow(dead_code)]
mod raw;
mod stream;
#[cfg(test)]
mod test;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

pub use model::{Capabilities, Model, ModelBuilder};
pub use openrouter::{File, Message, MessageToolCall, MessageToolResult, Openrouter};
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use raw::FinishReason;
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};
