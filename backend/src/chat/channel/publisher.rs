use std::future::Future;
use std::sync::Arc;

use tokio::sync::watch;

use super::Context;
use super::inner::Inner;
use super::mergeable::Mergeable;

/// Publisher for sending items to a channel.
///
/// Only one publisher can exist per channel ID at a time.
/// When dropped, the publisher clears the buffer and restores the sender for reconnection.
pub struct Publisher<S: Mergeable> {
    inner: Arc<Inner<S>>,
    ctx: Arc<Context<S>>,
    id: i32,
    sender: watch::Sender<()>,
}

impl<S: Mergeable + Clone + Send + 'static> Publisher<S> {
    pub(super) fn new(
        inner: Arc<Inner<S>>,
        ctx: Arc<Context<S>>,
        id: i32,
        sender: watch::Sender<()>,
    ) -> Self {
        Self {
            inner,
            ctx,
            id,
            sender,
        }
    }

    /// Publish a new item to the channel.
    ///
    /// The item will be merged with the last item in the buffer if possible.
    /// Otherwise, it will be appended as a new item.
    pub fn publish(&mut self, item: S) {
        let mut buffer = self.inner.buffer.lock().unwrap();
        if let Some(last) = buffer.last_mut() {
            if let Some(rest) = last.merge(item) {
                buffer.push(rest);
            }
        } else {
            buffer.push(item);
        }
        drop(buffer);
        self.sender.send_replace(());
    }

    /// Wait for a halt signal from subscribers.
    ///
    /// This allows publishers to be notified when they should stop publishing,
    /// typically used for graceful shutdown or when a client disconnects.
    pub fn wait_halt(&self) -> impl Future<Output = ()> + Send + 'static {
        let inner = self.inner.clone();
        async move {
            inner.stop_notify.notified().await;
        }
    }
}

impl<S: Mergeable + Clone> Drop for Publisher<S> {
    fn drop(&mut self) {
        // Clear the buffer to signal end of this publishing session
        let mut buffer = self.inner.buffer.lock().unwrap();
        buffer.clear();
        drop(buffer);

        // Remove from map - subscribers will get a fresh Inner when they reconnect
        self.ctx.map.lock().unwrap().remove(&self.id);

        // Dropping the sender closes the watch channel, notifying subscribers
    }
}
