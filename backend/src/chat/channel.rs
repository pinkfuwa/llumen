use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, Mutex, Weak},
};

use tokio::sync::{Notify, mpsc, watch};
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
    stop_notify: Notify,
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
            stop_notify: Notify::new(),
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

impl<S: Mergeable + Clone + Send + 'static> Publisher<S> {
    pub fn publish(&mut self, item: S) {
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
    pub fn wait_halt(&self) -> impl Future<Output = ()> + Send + 'static {
        let inner = self.inner.clone();
        async move {
            inner.stop_notify.notified().await;
        }
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
