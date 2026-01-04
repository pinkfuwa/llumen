mod error;
mod message;
mod model;
mod openrouter;
mod option;
#[allow(dead_code)]
mod raw;
mod stream;
mod stream_encode;
#[cfg(test)]
mod test;

static HTTP_REFERER: &str = "https://github.com/pinkfuwa/llumen";
static X_TITLE: &str = "llumen";

use std::io::Write;

pub use message::{File, Image, Message, MessageToolCall, MessageToolResult};
pub use model::{Capability, MaybeCapability, Model, ModelBuilder};
pub use openrouter::Openrouter;
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use raw::FinishReason;
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};

pub trait SyncStream {
    /// It does not return error
    /// Reads all data from the stream into the writer
    fn read(&mut self, writer: &mut dyn Write) -> usize;
}

impl SyncStream for Vec<u8> {
    fn read(&mut self, writer: &mut dyn Write) -> usize {
        let len = self.len();
        writer.write_all(self).ok();
        len
    }
}
