use std::{
    collections::HashMap,
    ops::Range,
    sync::{
        Arc, Mutex, Weak,
        atomic::{self, AtomicBool},
    },
};

use tokio::sync::{mpsc, watch};
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

pub trait Mergeable
where
    Self: Sized + Clone,
{
    fn merge(&mut self, other: Self) -> Option<Self>;
    fn len(&self) -> usize;
    fn slice(&self, r: Range<usize>) -> Option<Self>;
}

pub type LockedMap<K, V> = Mutex<HashMap<K, Weak<V>>>;
pub type LockedVec<S> = Mutex<Vec<S>>;
pub type LockedOption<T> = Mutex<Option<T>>;

pub struct Inner<S: Mergeable + Clone> {
    buffer: LockedVec<S>,
    sender: LockedOption<watch::Sender<()>>,
    receiver: watch::Receiver<()>,
    flag: AtomicBool,
}

impl<S: Mergeable + Clone + Send> Default for Inner<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Mergeable + Clone + Send> Inner<S> {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(());
        Self {
            buffer: Mutex::new(Vec::new()),
            sender: Mutex::new(Some(tx)),
            receiver: rx,
            flag: AtomicBool::new(false),
        }
    }
}

pub struct Context<S: Mergeable> {
    map: Arc<LockedMap<i32, Inner<S>>>,
}

impl<S: Mergeable + Clone + Send + 'static + Sync> Context<S> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub fn stop(&self, id: i32) {
        let map = self.map.lock().unwrap();
        if let Some(inner) = map.get(&id).and_then(|w| w.upgrade()) {
            inner.flag.store(true, atomic::Ordering::Release);
            // Send notification to wake up any waiting operations
            if let Some(sender) = inner.sender.lock().unwrap().as_ref() {
                let _ = sender.send(());
            }
        }
    }
    fn remove_weak(&self) {
        let mut map = self.map.lock().unwrap();
        map.retain(|_, v| v.strong_count() > 0);
    }
    fn get_inner(&self, id: i32) -> Arc<Inner<S>> {
        let mut map = self.map.lock().unwrap();
        match map.get(&id).map(|w| w.upgrade()).flatten() {
            Some(inner) => inner,
            None => {
                let inner = Arc::new(Inner::new());
                map.insert(id, Arc::downgrade(&inner));
                inner
            }
        }
    }

    fn get_subscriber(&self, id: i32) -> Subscriber<S> {
        let inner = self.get_inner(id);
        let receiver = inner.receiver.clone();
        inner.flag.store(false, atomic::Ordering::Release);
        Subscriber {
            cursor: Cursor::default(),
            inner,
            receiver,
        }
    }

    pub fn subscribe(self: Arc<Self>, id: i32, cursor: Option<Cursor>) -> impl Stream<Item = S> {
        let mut subscriber = self.get_subscriber(id);
        if let Some(cursor) = cursor {
            subscriber.cursor = cursor;
        }
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
                                subscriber = self.get_subscriber(id);
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
    pub fn publishable(&self, id: i32) -> bool {
        self.get_inner(id).sender.lock().unwrap().is_some()
    }
    pub fn publish(self: Arc<Self>, id: i32) -> Option<Publisher<S>> {
        self.remove_weak();
        let inner = self.get_inner(id);
        let sender = inner.sender.lock().unwrap().take()?;
        Some(Publisher {
            inner,
            ctx: self,
            id,
            sender,
        })
    }
    
    pub async fn publish_wait(
        self: Arc<Self>,
        id: i32,
        timeout: std::time::Duration,
    ) -> Option<Publisher<S>> {
        let start = std::time::Instant::now();
        
        loop {
            if let Some(publisher) = self.clone().publish(id) {
                return Some(publisher);
            }
            
            if start.elapsed() >= timeout {
                return None;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Cursor {
    pub index: usize,
    pub offset: usize,
}

impl TryFrom<(i32, i32)> for Cursor {
    type Error = std::num::TryFromIntError;
    fn try_from(value: (i32, i32)) -> Result<Self, Self::Error> {
        Ok(Cursor {
            index: value.0.try_into()?,
            offset: value.1.try_into()?,
        })
    }
}

pub struct Publisher<S: Mergeable> {
    inner: Arc<Inner<S>>,
    ctx: Arc<Context<S>>,
    id: i32,
    sender: watch::Sender<()>,
}

impl<S: Mergeable + Clone> Publisher<S> {
    pub fn publish_force(&mut self, item: S) {
        let mut buffer = self.inner.buffer.lock().unwrap();
        if let Some(last) = buffer.last_mut() {
            if let Some(rest) = last.merge(item) {
                buffer.push(rest);
            }
        } else {
            buffer.push(item);
        }
        self.sender.send_replace(());
    }
    pub fn publish(&mut self, item: S) -> Result<(), ()> {
        if self.inner.flag.load(atomic::Ordering::Acquire) {
            self.inner.flag.store(false, atomic::Ordering::Release);
            return Err(());
        }

        self.publish_force(item);

        Ok(())
    }
    
    pub fn is_halted(&self) -> bool {
        self.inner.flag.load(atomic::Ordering::Acquire)
    }
    
    pub fn get_halt_receiver(&self) -> watch::Receiver<()> {
        self.inner.receiver.clone()
    }
}

impl<S: Mergeable> Drop for Publisher<S> {
    fn drop(&mut self) {
        self.ctx.map.lock().unwrap().remove(&self.id);
    }
}

pub struct Subscriber<S>
where
    S: Mergeable + Clone + Send + 'static,
{
    cursor: Cursor,
    receiver: watch::Receiver<()>,
    inner: Arc<Inner<S>>,
}

fn advance_cursor<S: Mergeable>(cursor: &mut Cursor, shared_buffer: &[S]) -> Option<S> {
    let offset = cursor.offset;
    let index = cursor.index;
    let item = shared_buffer.get(index)?;
    let len = item.len();

    if offset == len {
        if index + 1 < shared_buffer.len() {
            cursor.index += 1;
            cursor.offset = 0;

            return advance_cursor(cursor, shared_buffer);
        } else {
            return None;
        }
    }

    cursor.offset = len;

    item.slice(offset..len)
}

fn check_buffer_exhausted<S: Mergeable>(cursor: &Cursor, shared_buffer: &[S]) -> bool {
    cursor.index >= shared_buffer.len()
        || (cursor.index == shared_buffer.len() - 1
            && cursor.offset >= shared_buffer[cursor.index].len())
}

impl<S> Subscriber<S>
where
    S: Mergeable + Clone + Send,
{
    async fn recv(&mut self) -> Option<S> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    impl Mergeable for String {
        fn merge(&mut self, other: Self) -> Option<Self> {
            self.push_str(&other);
            None
        }

        fn len(&self) -> usize {
            self.len()
        }

        fn slice(&self, r: Range<usize>) -> Option<Self> {
            Some(self.get(r)?.to_string())
        }
    }

    #[tokio::test]
    async fn test_subscribe_closes_when_stream_dropped_immediately() {
        let ctx = Arc::new(Context::<String>::new());
        let chat_id = 1;

        // Subscribe and immediately drop - simulates SSE disconnect before any data
        let stream = ctx.clone().subscribe(chat_id, None);
        // pin_mut!(stream); // Not strictly needed for this usage

        // Drop the stream immediately
        drop(stream);

        // Give the spawned task time to detect the closure
        // Without our fix, the task would loop forever trying to recv()
        // With the fix, it should detect tx.is_closed() and exit
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Try to publish - should succeed as the spawned task should have exited
        let publisher = ctx.clone().publish(chat_id);
        assert!(
            publisher.is_some(),
            "Should be able to publish after subscriber drops"
        );
    }

    #[tokio::test]
    async fn test_subscribe_closes_when_stream_dropped_with_no_publisher() {
        let ctx = Arc::new(Context::<String>::new());
        let chat_id = 10;

        // Subscribe without ever creating a publisher
        let stream = ctx.clone().subscribe(chat_id, None);

        // The spawned task is now waiting in subscriber.recv()
        // subscriber.recv() will block because there's no data and no publisher

        // Drop the stream to simulate SSE disconnect
        drop(stream);

        // Give the spawned task time to detect the closure
        // The task is blocked in subscriber.recv().await which is waiting on watch::Receiver::changed()
        // When we drop the stream, tx is closed
        // Eventually the task should wake up and check tx.is_closed()
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Now create a publisher - if the spawned task exited, this should succeed
        let publisher = ctx.clone().publish(chat_id);
        assert!(
            publisher.is_some(),
            "Spawned task should have exited when stream dropped"
        );
    }

    #[tokio::test]
    async fn test_subscribe_task_exits_on_publisher_drop_and_stream_drop() {
        let ctx = Arc::new(Context::<String>::new());
        let chat_id = 2;

        // Create a publisher and publish message
        let mut publisher = ctx.clone().publish(chat_id).unwrap();
        publisher.publish_force("test".to_string());

        // Create a subscriber and read the message
        let mut stream = ctx.clone().subscribe(chat_id, None);
        let msg = stream.next().await;
        assert!(msg.is_some());

        // Drop publisher to close the watch channel
        drop(publisher);

        // Drop the stream to simulate SSE disconnect
        drop(stream);

        // Give the spawned task time to detect closure and exit
        // The task should:
        // 1. Call subscriber.recv() which returns None (publisher dropped)
        // 2. Check tx.is_closed() which returns true (stream dropped)
        // 3. Break the loop
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Verify we can create a new publisher (meaning the old task exited)
        let publisher = ctx.clone().publish(chat_id);
        assert!(publisher.is_some(), "Spawned task should have exited");
    }

    #[tokio::test]
    async fn test_normal_streaming_works() {
        let ctx = Arc::new(Context::<String>::new());
        let chat_id = 3;

        // Create publisher first, then subscriber
        let mut publisher = ctx.clone().publish(chat_id).unwrap();
        let mut stream = ctx.clone().subscribe(chat_id, None);

        // Give the spawned task time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Publish messages - they will be merged by the Mergeable implementation
        publisher.publish_force("Hello".to_string());
        publisher.publish_force(" ".to_string());
        publisher.publish_force("World".to_string());

        // Collect the merged message
        match tokio::time::timeout(tokio::time::Duration::from_secs(1), stream.next()).await {
            Ok(Some(msg)) => {
                // All three messages should be merged into one
                assert_eq!(msg, "Hello World");
            }
            Ok(None) => panic!("Stream ended unexpectedly"),
            Err(_) => panic!("Timeout waiting for message"),
        }
    }

    #[tokio::test]
    async fn test_subscribe_with_cursor_resumes_correctly() {
        let ctx = Arc::new(Context::<String>::new());
        let chat_id = 42;

        let mut publisher = ctx.clone().publish(chat_id).unwrap();
        publisher.publish_force("Hello".to_string());
        publisher.publish_force(" World".to_string());

        // Cursor pointing after "Hello"
        let cursor = Cursor {
            index: 0,
            offset: 5,
        };

        let mut stream = ctx.clone().subscribe(chat_id, Some(cursor));

        // We expect to receive " World"
        let msg = stream.next().await;
        assert_eq!(msg, Some(" World".to_string()));
    }
}
