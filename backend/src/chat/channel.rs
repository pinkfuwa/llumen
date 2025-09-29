use std::{
    cmp,
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, Mutex, Weak,
        atomic::{self, AtomicBool},
    },
};

use futures_util::Stream;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;

pub trait Mergeable
where
    Self: Sized + Clone,
{
    fn merge(&mut self, other: Self) -> Option<Self>;
    fn len(&self) -> usize;
    /// get a split part from the end
    fn split_end(&self, split: usize) -> Option<Self>;
    /// get a split part from the start
    fn split_start(&self, split: usize) -> Self;
}

pub type LockedMap<K, V> = Mutex<HashMap<K, Weak<V>>>;
pub type LockedVec<S> = Mutex<Vec<S>>;
pub type LockedOption<T> = Mutex<Option<T>>;

pub struct Inner<S: Mergeable + Clone> {
    buffer: LockedVec<S>,
    sender: LockedOption<watch::Sender<Cursor>>,
    flag: AtomicBool,
}

impl<S: Mergeable + Clone + Send> Default for Inner<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Mergeable + Clone + Send> Inner<S> {
    pub fn new() -> Self {
        let (tx, _) = watch::channel(Cursor {
            index: 0,
            offset: 0,
        });
        Self {
            buffer: Mutex::new(Vec::new()),
            sender: Mutex::new(Some(tx)),
            flag: AtomicBool::new(false),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    index: usize,
    offset: usize,
}

impl cmp::PartialOrd for Cursor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.index.cmp(&other.index) {
            cmp::Ordering::Equal => Some(self.offset.cmp(&other.offset)),
            ord => Some(ord),
        }
    }
}

impl Cursor {
    pub fn new(index: usize, offset: usize) -> Self {
        Self { index, offset }
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
    // FIXME: halt is not immediate
    pub fn stop(&self, id: i32) {
        let map = self.map.lock().unwrap();
        if let Some(inner) = map.get(&id).and_then(|w| w.upgrade()) {
            inner.flag.store(true, atomic::Ordering::Release);
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
        let receiver = inner.sender.lock().unwrap().as_ref().unwrap().subscribe();
        Subscriber {
            from: Cursor::new(0, 0),
            to: Cursor::new(0, 0),
            inner,
            receiver,
            ended: false,
        }
    }
    pub fn subscribe(self: Arc<Self>, id: i32) -> impl Stream<Item = S> {
        let mut subscriber = self.get_subscriber(id);
        let (tx, rx) = mpsc::channel::<S>(1);

        tokio::spawn(async move {
            loop {
                match subscriber.recv().await {
                    Some(item) if item.len() == 0 => {
                        continue;
                    }
                    Some(item) => {
                        if tx.send(item).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        subscriber = self.get_subscriber(id);
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }
    pub fn publish(self: Arc<Self>, id: i32) -> Publisher<S> {
        self.remove_weak();
        let inner = self.get_inner(id);
        Publisher {
            inner,
            ctx: self,
            id,
        }
    }
}

pub struct Publisher<S: Mergeable> {
    inner: Arc<Inner<S>>,
    ctx: Arc<Context<S>>,
    id: i32,
}

impl<S: Mergeable + Clone + Debug> Publisher<S> {
    pub fn publish_force(&mut self, item: S) {
        let mut buffer = self.inner.buffer.lock().unwrap();
        if let Some(last) = buffer.last_mut() {
            if let Some(rest) = last.merge(item) {
                buffer.push(rest);
            }
        } else {
            buffer.push(item);
        }
        let index = match buffer.is_empty() {
            true => 0,
            false => buffer.len() - 1,
        };
        let offset = buffer.last().map(|s| s.len()).unwrap_or(0);
        let cursor = Cursor::new(index, offset);
        self.inner
            .sender
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .send_replace(cursor);
    }
    /// Error when the channel is closed
    pub fn publish(&mut self, item: S) -> Result<(), ()> {
        if self.inner.flag.load(atomic::Ordering::Acquire) {
            self.inner.flag.store(false, atomic::Ordering::Release);
            return Err(());
        }

        self.publish_force(item);

        Ok(())
    }
}

impl<S: Mergeable> Drop for Publisher<S> {
    fn drop(&mut self) {
        let sender = self.inner.sender.lock().unwrap().take();
        drop(sender);
        self.ctx.map.lock().unwrap().remove(&self.id);
    }
}

pub struct Subscriber<S>
where
    S: Mergeable + Clone + Send + 'static,
{
    from: Cursor,
    to: Cursor,
    receiver: watch::Receiver<Cursor>,
    inner: Arc<Inner<S>>,
    ended: bool,
}

impl<S> Subscriber<S>
where
    S: Mergeable + Clone + Send,
{
    /// skip all None
    fn advance_cursor(&self, shared_buffer: &[S]) -> (Cursor, Option<S>) {
        if self.from == self.to {
            return (self.to.clone(), None);
        }
        let from_index = self.from.index;
        let from_offset = self.from.offset;

        match from_index.cmp(&self.to.index) {
            cmp::Ordering::Equal => (
                self.to.clone(),
                shared_buffer[from_index]
                    .split_end(from_offset)
                    .map(|x| x.split_start(self.to.offset - from_offset)),
            ),
            cmp::Ordering::Less if from_offset == 0 => (
                Cursor::new(from_index + 1, 0),
                Some(shared_buffer[from_index].clone()),
            ),
            cmp::Ordering::Less => (
                Cursor::new(from_index + 1, 0),
                shared_buffer[from_index].split_end(from_offset),
            ),
            cmp::Ordering::Greater => panic!("from_index should not be greater than to_index"),
        }
    }
    async fn recv(&mut self) -> Option<S> {
        loop {
            {
                let update = self.receiver.borrow_and_update();
                self.to = *update;
            }

            let (new_cursor, item) = {
                let shared_buffer = self.inner.buffer.lock().unwrap();
                self.advance_cursor(&shared_buffer)
            };

            log::trace!(
                "recv: from: {:?}, to: {:?}, new_cursor: {:?}",
                self.from,
                self.to,
                new_cursor
            );

            self.from = new_cursor;

            if item.is_some() {
                return item;
            }

            if self.ended {
                return None;
            }

            if self.receiver.changed().await.is_err() {
                self.ended = true;
            }
        }
    }
}
