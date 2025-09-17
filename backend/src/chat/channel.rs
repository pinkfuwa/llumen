use std::{
    collections::HashMap,
    pin::Pin,
    sync::{
        Arc, Mutex, Weak,
        atomic::{self, AtomicBool},
    },
    task,
};

use futures_util::{FutureExt, Stream, pin_mut};
use tokio::sync::watch;

pub type LockedMap<K, V> = Arc<Mutex<HashMap<K, Weak<V>>>>;
pub type LockedVec<S> = Arc<Mutex<Vec<S>>>;

pub struct Context<S: Mergeable> {
    map: LockedMap<i32, Inner<S>>,
}

impl<S: Mergeable + Clone> Context<S> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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
    pub fn subscribe(&self, id: i32) -> Subscriber<S> {
        self.get_inner(id).subscribe()
    }
    pub fn publish(&self, id: i32) -> Publisher<S> {
        self.remove_weak();
        self.get_inner(id).publish()
    }
}

pub struct Inner<S: Mergeable + Clone> {
    buffer: LockedVec<S>,
    last_signal: (
        Mutex<Option<watch::Sender<Cursor>>>,
        watch::Receiver<Cursor>,
    ),
    flag: Arc<AtomicBool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cursor {
    index: usize,
    offset: usize,
}

impl Cursor {
    pub fn new(index: usize, offset: usize) -> Self {
        Self { index, offset }
    }
}

pub struct Subscriber<S: Mergeable> {
    cursor: Cursor,
    buffer: LockedVec<S>,
    rx: watch::Receiver<Cursor>,
}

pub struct Publisher<S: Mergeable> {
    sender: watch::Sender<Cursor>,
    flag: Arc<AtomicBool>,
    buffer: LockedVec<S>,
}

pub trait Mergeable
where
    Self: Sized + Clone,
{
    fn merge(self, other: Self) -> (Self, Option<Self>);
    fn len(&self) -> usize;
    /// get a split part from the end
    fn split_end(&self, split: usize) -> Option<Self>;
    /// get a split part from the start
    fn split_start(&self, split: usize) -> Self;
}

impl Mergeable for String {
    fn merge(self, other: Self) -> (Self, Option<Self>) {
        let mut s = self;
        s.push_str(&other);
        match s.len() > 4096 {
            true => {
                let rest = s.split_off(2048);
                (s, Some(rest))
            }
            false => (s, None),
        }
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn split_end(&self, split: usize) -> Option<Self> {
        if self.len() < split {
            None
        } else {
            Some(self[self.len() - split..].to_owned())
        }
    }

    fn split_start(&self, split: usize) -> Self {
        match self.len() <= split {
            true => self.clone(),
            false => self[..split].to_owned(),
        }
    }
}

pub trait SplitCollection {
    fn slice(&self, from: &Cursor, to: &Cursor) -> Self;
}

impl<T: Mergeable> SplitCollection for Vec<T> {
    fn slice(&self, from: &Cursor, to: &Cursor) -> Vec<T> {
        let mut result = self[from.index..=to.index].to_vec();
        if let Some(first) = result.first_mut() {
            *first = first
                .split_end(from.offset)
                .unwrap_or_else(|| first.clone());
        }
        if let Some(last) = result.last_mut() {
            *last = last.split_start(to.offset);
        }
        return result;
    }
}

impl<S: Mergeable + Clone> Default for Inner<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Mergeable + Clone> Inner<S> {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(Cursor {
            index: 0,
            offset: 0,
        });
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            last_signal: (Mutex::new(Some(tx)), rx),
            flag: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn subscribe(&self) -> Subscriber<S> {
        Subscriber {
            cursor: Cursor::new(0, 0),
            buffer: self.buffer.clone(),
            rx: self.last_signal.1.clone(),
        }
    }
    pub fn publish(&self) -> Publisher<S> {
        Publisher {
            sender: self.last_signal.0.lock().unwrap().take().unwrap(),
            flag: self.flag.clone(),
            buffer: self.buffer.clone(),
        }
    }
}

impl<S: Mergeable + Clone> Publisher<S> {
    /// Error when the channel is closed
    pub fn publish(&mut self, item: S) -> Result<(), ()> {
        if self.flag.load(atomic::Ordering::Acquire) {
            self.flag.store(false, atomic::Ordering::Release);
            return Err(());
        }

        let mut buffer = self.buffer.lock().unwrap();
        if let Some(last) = buffer.last_mut() {
            let (new_last, rest) = last.clone().merge(item);
            *last = new_last;
            if let Some(rest) = rest {
                buffer.push(rest);
            }
        } else {
            buffer.push(item);
        }
        let index = if buffer.is_empty() {
            0
        } else {
            buffer.len() - 1
        };
        let offset = buffer.last().map(|s| s.len()).unwrap_or(0);
        let cursor = Cursor::new(index, offset);
        self.sender.send(cursor).ok();

        Ok(())
    }
}

impl<S: Mergeable + Clone> Subscriber<S> {
    async fn recv(&mut self) -> Option<Vec<S>> {
        while let Ok(_) = self.rx.changed().await {
            let new_cursor = self.rx.borrow().to_owned();
            if new_cursor == self.cursor {
                continue;
            }

            let buffer = self.buffer.lock().unwrap();
            let items = buffer.slice(&self.cursor, &new_cursor);

            self.cursor = new_cursor;

            return Some(items);
        }

        let buffer = self.buffer.lock().unwrap();
        let mut items = buffer[self.cursor.index..].to_vec();
        if let Some(last) = items.last_mut() {
            if let Some(new_last) = last.split_end(self.cursor.offset) {
                *last = new_last;
            }
        }

        if items.is_empty() {
            return None;
        }

        return Some(items);
    }
    pub fn flatten(self) -> StreamSubscriber<S>
    where
        S: Send,
    {
        StreamSubscriber {
            inner: self,
            buffer: Vec::new(),
        }
    }
}

pub struct StreamSubscriber<S>
where
    S: Mergeable + Clone + Send + 'static,
{
    inner: Subscriber<S>,
    buffer: Vec<S>,
}

impl<S> Stream for StreamSubscriber<S>
where
    S: Mergeable + Clone + Send + 'static + Unpin,
{
    type Item = S;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let buffer = &mut this.buffer;
        let inner = &mut this.inner;

        if let Some(item) = buffer.pop() {
            return task::Poll::Ready(Some(item));
        }

        let fut = inner.recv();
        let results = futures_util::ready!(Box::pin(fut).poll_unpin(cx));
        match results {
            Some(mut items) => {
                items.reverse();
                if let Some(item) = items.pop() {
                    *buffer = items;
                    task::Poll::Ready(Some(item))
                } else {
                    task::Poll::Ready(None)
                }
            }
            None => task::Poll::Ready(None),
        }
    }
}
