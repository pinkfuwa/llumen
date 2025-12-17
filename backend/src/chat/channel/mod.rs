use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use tokio::sync::mpsc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

mod inner;
mod mergeable;
mod publisher;
mod subscriber;

pub use inner::Cursor;
pub use mergeable::Mergeable;
pub use publisher::Publisher;
pub use subscriber::Subscriber;

use inner::Inner;

#[cfg(test)]
mod tests;

pub type LockedMap<K, V> = Mutex<HashMap<K, Weak<V>>>;

/// Context manages multiple channels, each identified by an i32 ID.
///
/// The context maintains weak references to channel inner state, allowing
/// channels to be automatically cleaned up when no publishers or subscribers exist.
pub struct Context<S: Mergeable> {
    map: Arc<LockedMap<i32, Inner<S>>>,
}

impl<S: Mergeable + Clone + Send + 'static + Sync> Context<S> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Stop a channel by notifying all publishers to halt.
    ///
    /// This will wait until the channel is fully closed (all publishers dropped).
    pub async fn stop(&self, id: i32) {
        let maybe_inner = {
            let map = self.map.lock().unwrap();
            map.get(&id).and_then(|w| w.upgrade())
        };
        if let Some(inner) = maybe_inner {
            let mut receiver = inner.receiver.clone();
            inner.stop_notify.notify_waiters();
            while receiver.changed().await.is_ok() {
                inner.stop_notify.notify_waiters();
            }
        }
    }

    /// Remove weak references that no longer have strong references.
    fn remove_weak(&self) {
        let mut map = self.map.lock().unwrap();
        map.retain(|_, v| v.strong_count() > 0);
    }

    /// Get or create the inner state for a channel.
    fn get_inner(&self, id: i32) -> Arc<Inner<S>> {
        let mut map = self.map.lock().unwrap();
        match map.get(&id).and_then(|w| w.upgrade()) {
            Some(inner) => inner,
            None => {
                let inner = Arc::new(Inner::new());
                map.insert(id, Arc::downgrade(&inner));
                inner
            }
        }
    }

    /// Get a subscriber for a channel.
    fn get_subscriber(&self, id: i32, cursor: Cursor) -> Subscriber<S> {
        let inner = self.get_inner(id);
        Subscriber::new(inner, cursor)
    }

    /// Subscribe to a channel, returning a stream of items.
    ///
    /// The stream will automatically reconnect if the channel is recreated.
    /// If a cursor is provided, the stream will start from that position.
    pub fn subscribe(self: Arc<Self>, id: i32, cursor: Option<Cursor>) -> impl Stream<Item = S> {
        let mut subscriber = self.get_subscriber(id, cursor.unwrap_or_default());
        let (tx, rx) = mpsc::channel::<S>(1);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    recv_result = subscriber.recv() => {
                        match recv_result {
                            Some(item) if item.len() == 0 => continue,
                            None => {
                                if tx.is_closed() {
                                    break;
                                }
                                subscriber = self.get_subscriber(id, Cursor::default());
                            }
                            Some(item) => {
                                if tx.send(item).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    _ = tx.closed() => {
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }

    /// Check if a channel can be published to.
    ///
    /// Returns `false` if a publisher already exists for this channel.
    pub fn publishable(&self, id: i32) -> bool {
        self.get_inner(id).sender.lock().unwrap().is_some()
    }

    /// Create a publisher for a channel.
    ///
    /// Returns `None` if a publisher already exists for this channel.
    pub fn publish(self: Arc<Self>, id: i32) -> Option<Publisher<S>> {
        self.remove_weak();
        let inner = self.get_inner(id);
        let sender = inner.sender.lock().unwrap().take()?;
        Some(Publisher::new(inner, self, id, sender))
    }
}

impl<S: Mergeable + Clone + Send + 'static + Sync> Default for Context<S> {
    fn default() -> Self {
        Self::new()
    }
}
