mod error;
mod message;
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

pub use message::{File, Image, Message, MessageToolCall, MessageToolResult};
pub use model::{Capability, MaybeCapability, Model, ModelBuilder};
pub use openrouter::Openrouter;
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use raw::FinishReason;
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};
