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

pub use message::{File, Image, Message, MessageToolCall, MessageToolResult};
pub use model::{Capability, MaybeCapability, Model, ModelBuilder};
pub use openrouter::Openrouter;
pub use option::{CompletionOption, ReasoningEffort, Tool};
pub use raw::FinishReason;
pub use stream::{StreamCompletion, StreamCompletionResp, ToolCall};

pub trait SyncStream {
    /// Read next chunk into buffer, returns number of bytes read
    /// Returns 0 when no more data available
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize;

    /// Get total size of the stream
    fn len(&self) -> usize;

    /// Check if stream is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[allow(dead_code)]
pub struct VecStream {
    data: Vec<u8>,
    position: usize,
}

#[allow(dead_code)]
impl VecStream {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }
}

impl SyncStream for VecStream {
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize {
        let remaining = self.data.len().saturating_sub(self.position);
        let to_read = std::cmp::min(buf.len(), remaining);

        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self.data[self.position..self.position + to_read]);
            self.position += to_read;
        }

        to_read
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// For backward compatibility
impl SyncStream for Vec<u8> {
    fn read_chunk(&mut self, buf: &mut [u8]) -> usize {
        let to_read = std::cmp::min(buf.len(), self.len());
        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self[..to_read]);
            self.drain(..to_read);
        }
        to_read
    }

    fn len(&self) -> usize {
        self.len()
    }
}
