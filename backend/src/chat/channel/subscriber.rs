use std::sync::Arc;

use tokio::sync::watch;

use super::inner::{Cursor, Inner, advance_cursor, check_buffer_exhausted};
use super::mergeable::Mergeable;

/// Subscriber for receiving items from a channel.
///
/// Subscribers maintain their own cursor position in the buffer and can read
/// items independently without blocking other subscribers.
pub struct Subscriber<S>
where
    S: Mergeable + Clone + Send + 'static,
{
    cursor: Cursor,
    receiver: watch::Receiver<()>,
    inner: Arc<Inner<S>>,
}

impl<S> Subscriber<S>
where
    S: Mergeable + Clone + Send,
{
    pub(super) fn new(inner: Arc<Inner<S>>, cursor: Cursor) -> Self {
        let receiver = inner.receiver.clone();
        Self {
            cursor,
            receiver,
            inner,
        }
    }

    /// Receive the next item from the channel.
    ///
    /// Returns:
    /// - `Some(item)` if an item is available
    /// - `None` if the channel is closed and all items have been consumed
    ///
    /// This method will wait for new items if the buffer is exhausted but the channel
    /// is still open.
    pub async fn recv(&mut self) -> Option<S> {
        loop {
            let item = {
                let buffer = self.inner.buffer.lock().unwrap();
                advance_cursor(&mut self.cursor, &buffer)
            };

            if let Some(item) = item {
                return Some(item);
            }

            if self.receiver.changed().await.is_err() {
                let buffer = self.inner.buffer.lock().unwrap();

                if check_buffer_exhausted(&self.cursor, &buffer) {
                    return None;
                }
            }
        }
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Set the cursor position.
    ///
    /// This allows resuming from a specific position in the buffer.
    pub fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    /// Clone the receiver to create a new subscriber at the same position.
    pub fn clone_at_position(&self) -> Self {
        Self {
            cursor: self.cursor,
            receiver: self.receiver.clone(),
            inner: self.inner.clone(),
        }
    }
}
