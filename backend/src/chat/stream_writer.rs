use crate::chat::channel::Publisher;
use crate::chat::token::Token;

/// StreamWriter wraps the Publisher to provide a cleaner interface for
/// strategies.
///
/// **Purpose**: Abstract HOW tokens are sent to client.
///
/// **Today's limitation**: Due to Rust lifetime complexity with async +
/// borrowing, this is currently only used as documentation. The actual
/// implementation still uses `CompletionSession::put_stream()` directly.
///
/// **Future**: When we add multiplexing (concurrent streams per chat),
/// StreamWriter will become the primary interface with methods like
/// `write_to_chunk(chunk_id, token)`.
///
/// **Why it exists**: Shows the architectural intent even if not fully used
/// yet. Strategies SHOULD use StreamWriter conceptually, even if implementation
/// uses Publisher.
pub struct StreamWriter<'a> {
    publisher: &'a mut Publisher<Token>,
}

impl<'a> StreamWriter<'a> {
    /// Creates a StreamWriter wrapping the given publisher.
    pub fn new(publisher: &'a mut Publisher<Token>) -> Self {
        Self { publisher }
    }

    /// Writes a single token immediately to the stream.
    pub fn write(&mut self, token: Token) {
        self.publisher.publish(token);
    }
}
