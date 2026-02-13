use anyhow::Result;
use futures_util::Stream;

use super::Token;
use super::context::StreamEndReason;

/// Abstraction for where tokens are sent during completion.
///
/// This allows DeepAgent and other subagents to stream tokens without
/// being tightly coupled to CompletionSession. Different implementations
/// can send tokens via SSE, collect them for testing, or send via MCP.
pub trait TokenSink {
    /// Add a single token to the stream.
    fn add_token(&mut self, token: Token);

    /// Stream tokens from the given async stream.
    /// Returns the reason the stream ended (completed, halted, or error).
    fn put_stream<E: std::error::Error + Send + 'static>(
        &mut self,
        stream: impl Stream<Item = Result<Token, E>> + Unpin + Send,
    ) -> impl std::future::Future<Output = Result<StreamEndReason>> + Send;

    /// Update usage statistics (cost and token count).
    fn update_usage(&mut self, cost: f32, tokens: i32);
}
