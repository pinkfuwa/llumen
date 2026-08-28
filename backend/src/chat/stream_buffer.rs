use std::{
    collections::HashMap,
    future::Future,
    ops::Range,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, Weak},
    task::{Context as TaskContext, Poll, Waker},
};

use tokio_stream::Stream;

/// Defines how chat stream tokens merge and slice across buffered chunks.
pub trait Mergeable
where
    Self: Sized + Clone,
{
    fn merge(&mut self, other: Self) -> Option<Self>;
    fn len(&self) -> usize;
    fn slice(&self, r: Range<usize>) -> Option<Self>;
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct State<S> {
    buffer: Vec<S>,
    subscriber_wakers: Vec<Option<Waker>>,
    free_slots: Vec<usize>,
    halt_waker: Option<Waker>,
    publisher_drop_waker: Option<Waker>,
    halted: bool,
    publisher_alive: bool,
    publisher_closed: bool,
}

impl<S> State<S> {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            subscriber_wakers: Vec::new(),
            free_slots: Vec::new(),
            halt_waker: None,
            publisher_drop_waker: None,
            halted: false,
            publisher_alive: false,
            publisher_closed: false,
        }
    }

    fn allocate_slot(&mut self) -> usize {
        if let Some(slot) = self.free_slots.pop() {
            if let Some(entry) = self.subscriber_wakers.get_mut(slot) {
                *entry = None;
                return slot;
            }
        }
        let slot = self.subscriber_wakers.len();
        self.subscriber_wakers.push(None);
        slot
    }

    fn take_subscriber_wakers(&mut self) -> Vec<Waker> {
        self.subscriber_wakers
            .iter_mut()
            .filter_map(Option::take)
            .collect()
    }
}

fn wake_all(wakers: Vec<Waker>) {
    for waker in wakers {
        waker.wake();
    }
}

struct Inner<S> {
    state: Mutex<State<S>>,
}

impl<S> Inner<S> {
    fn new() -> Self {
        Self {
            state: Mutex::new(State::new()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State<S>> {
        lock_mutex(&self.state)
    }
}

/// Manages channel lifecycles for streaming chat tokens.
pub struct Context<S: Mergeable> {
    map: Arc<Mutex<HashMap<i32, Weak<Inner<S>>>>>,
}

impl<S: Mergeable + Clone + Send + 'static + Sync> Context<S> {
    /// Creates a new channel context ready to produce publishers/subscribers.
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Signals a stop to all halt waiters for the given channel id, then waits
    /// until the publisher (if any) has dropped.
    pub async fn stop(&self, id: i32) {
        let Some(inner) = self.upgrade_inner(id) else {
            return;
        };

        let halt_waker = {
            let mut state = inner.lock();
            state.halted = true;
            state.halt_waker.take()
        };
        if let Some(waker) = halt_waker {
            waker.wake();
        }

        PublisherDropWait { inner }.await;
    }

    fn upgrade_inner(&self, id: i32) -> Option<Arc<Inner<S>>> {
        let map = lock_mutex(&self.map);
        map.get(&id).and_then(|weak_inner| weak_inner.upgrade())
    }

    /// Cleans up dead entries to prevent unbounded map growth.
    fn remove_weak(&self) {
        let mut map = lock_mutex(&self.map);
        map.retain(|_, weak_inner| weak_inner.strong_count() > 0);
    }

    /// Ensures there is an `Inner` for the channel, creating one when
    /// necessary.
    fn get_inner(&self, id: i32) -> Arc<Inner<S>> {
        loop {
            let existing = {
                let map = lock_mutex(&self.map);
                map.get(&id).and_then(|weak_inner| weak_inner.upgrade())
            };
            if let Some(inner) = existing
                && !inner.lock().publisher_closed
            {
                return inner;
            }

            let mut map = lock_mutex(&self.map);
            if let Some(inner) = map.get(&id).and_then(|weak_inner| weak_inner.upgrade()) {
                drop(map);
                if !inner.lock().publisher_closed {
                    return inner;
                }
                continue;
            }
            let inner = Arc::new(Inner::new());
            map.insert(id, Arc::downgrade(&inner));
            return inner;
        }
    }

    fn get_subscriber(self: &Arc<Self>, id: i32) -> Subscriber<S> {
        let inner = self.get_inner(id);
        let slot = inner.lock().allocate_slot();
        Subscriber {
            cursor: Cursor::default(),
            inner,
            slot,
            ctx: Arc::clone(self),
            id,
        }
    }

    /// Starts streaming tokens for a subscriber, optionally resuming from a
    /// cursor.
    pub fn subscribe(self: Arc<Self>, id: i32, cursor: Option<Cursor>) -> impl Stream<Item = S> {
        let mut subscriber = self.get_subscriber(id);
        if let Some(cursor) = cursor {
            subscriber.cursor = cursor;
        }
        SubscribeStream {
            ctx: self,
            id,
            subscriber,
        }
    }

    /// Returns true when a new publisher can be created for the channel.
    pub fn publishable(&self, id: i32) -> bool {
        let Some(inner) = self.upgrade_inner(id) else {
            return true;
        };
        let state = inner.lock();
        !state.publisher_alive || state.publisher_closed
    }

    fn unregister_idle_inner(&self, id: i32, inner: &Arc<Inner<S>>) {
        if inner.lock().publisher_alive {
            return;
        }
        let mut map = lock_mutex(&self.map);
        let this = Arc::downgrade(inner);
        if !map
            .get(&id)
            .is_some_and(|existing| Weak::ptr_eq(existing, &this))
        {
            return;
        }
        if Arc::strong_count(inner) == 1 {
            map.remove(&id);
        }
    }

    /// Claims the channel to become the single publisher that writes into it.
    pub fn publish(self: Arc<Self>, id: i32) -> Option<Publisher<S>> {
        self.remove_weak();
        let inner = self.get_inner(id);
        {
            let mut state = inner.lock();
            if state.publisher_alive {
                return None;
            }
            state.publisher_alive = true;
            state.publisher_closed = false;
            state.halted = false;
        }
        Some(Publisher {
            inner,
            ctx: self,
            id,
        })
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
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
}

impl<S: Mergeable + Clone + Send + 'static> Publisher<S> {
    /// Appends a token for subscribers, merging with the previous chunk when
    /// possible.
    pub fn publish(&mut self, item: S) {
        let wakers = {
            let mut state = self.inner.lock();
            if let Some(last) = state.buffer.last_mut() {
                if let Some(rest) = last.merge(item) {
                    state.buffer.push(rest);
                }
            } else {
                state.buffer.push(item);
            }
            state.take_subscriber_wakers()
        };
        wake_all(wakers);
    }

    /// Returns a future that resolves once the publisher is halted.
    pub fn wait_halt(&self) -> impl Future<Output = ()> + Send + 'static {
        WaitHalt {
            inner: self.inner.clone(),
        }
    }
}

impl<S: Mergeable> Drop for Publisher<S> {
    fn drop(&mut self) {
        let (subscriber_wakers, publisher_drop_waker) = {
            let mut state = self.inner.lock();
            state.publisher_alive = false;
            state.publisher_closed = true;
            let subscriber_wakers = state.take_subscriber_wakers();
            let publisher_drop_waker = state.publisher_drop_waker.take();
            (subscriber_wakers, publisher_drop_waker)
        };
        {
            let mut map = lock_mutex(&self.ctx.map);
            let this = Arc::downgrade(&self.inner);
            if map
                .get(&self.id)
                .is_some_and(|weak_inner| Weak::ptr_eq(weak_inner, &this))
            {
                map.remove(&self.id);
            }
        }
        wake_all(subscriber_wakers);
        if let Some(waker) = publisher_drop_waker {
            waker.wake();
        }
    }
}

struct Subscriber<S: Mergeable + Clone + Send + Sync + 'static> {
    cursor: Cursor,
    inner: Arc<Inner<S>>,
    slot: usize,
    ctx: Arc<Context<S>>,
    id: i32,
}

impl<S: Mergeable + Clone + Send + Sync + 'static> Drop for Subscriber<S> {
    fn drop(&mut self) {
        {
            let mut state = self.inner.lock();
            if let Some(slot) = state.subscriber_wakers.get_mut(self.slot) {
                *slot = None;
            }
            state.free_slots.push(self.slot);
        }
        self.ctx.unregister_idle_inner(self.id, &self.inner);
    }
}

struct SubscribeStream<S: Mergeable + Clone + Send + Sync + 'static> {
    ctx: Arc<Context<S>>,
    id: i32,
    subscriber: Subscriber<S>,
}

impl<S> Stream for SubscribeStream<S>
where
    S: Mergeable + Clone + Send + Sync + 'static,
{
    type Item = S;

    fn poll_next(self: Pin<&mut Self>, task: &mut TaskContext<'_>) -> Poll<Option<S>> {
        let this = self.get_mut();
        loop {
            match this.subscriber.poll_recv(task) {
                Poll::Ready(Some(item)) if item.len() == 0 => continue,
                Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                Poll::Ready(None) => {
                    this.subscriber = this.ctx.get_subscriber(this.id);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

fn advance_cursor<S: Mergeable>(cursor: &mut Cursor, shared_buffer: &[S]) -> Option<S> {
    if shared_buffer.is_empty() {
        return None;
    }
    if cursor.index >= shared_buffer.len() {
        let last_index = shared_buffer.len() - 1;
        cursor.index = last_index;
        cursor.offset = shared_buffer
            .get(last_index)
            .map(Mergeable::len)
            .unwrap_or(0);
        return None;
    }

    loop {
        let item = shared_buffer.get(cursor.index)?;
        let len = item.len();

        if cursor.offset > len {
            cursor.offset = len;
        }

        if cursor.offset == len {
            let next_index = cursor.index.saturating_add(1);
            if next_index < shared_buffer.len() {
                cursor.index = next_index;
                cursor.offset = 0;
                continue;
            }
            return None;
        }

        match item.slice(cursor.offset..len) {
            Some(sliced) => {
                cursor.offset = len;
                return Some(sliced);
            }
            None => {
                let next_index = cursor.index.saturating_add(1);
                if next_index < shared_buffer.len() {
                    cursor.index = next_index;
                    cursor.offset = 0;
                    continue;
                }
                return None;
            }
        }
    }
}

impl<S> Subscriber<S>
where
    S: Mergeable + Clone + Send + Sync + 'static,
{
    fn poll_recv(&mut self, task: &mut TaskContext<'_>) -> Poll<Option<S>> {
        let mut state = self.inner.lock();
        if let Some(item) = advance_cursor(&mut self.cursor, &state.buffer) {
            return Poll::Ready(Some(item));
        }
        if state.publisher_closed {
            return Poll::Ready(None);
        }
        if let Some(slot) = state.subscriber_wakers.get_mut(self.slot) {
            *slot = Some(task.waker().clone());
        }
        Poll::Pending
    }
}

struct WaitHalt<S> {
    inner: Arc<Inner<S>>,
}

impl<S> Future for WaitHalt<S> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, task: &mut TaskContext<'_>) -> Poll<()> {
        let mut state = self.inner.lock();
        if state.halted {
            state.halt_waker = None;
            return Poll::Ready(());
        }
        state.halt_waker = Some(task.waker().clone());
        Poll::Pending
    }
}

impl<S> Drop for WaitHalt<S> {
    fn drop(&mut self) {
        self.inner.lock().halt_waker = None;
    }
}

struct PublisherDropWait<S> {
    inner: Arc<Inner<S>>,
}

impl<S> Future for PublisherDropWait<S> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, task: &mut TaskContext<'_>) -> Poll<()> {
        let mut state = self.inner.lock();
        if !state.publisher_alive {
            state.publisher_drop_waker = None;
            return Poll::Ready(());
        }
        state.publisher_drop_waker = Some(task.waker().clone());
        Poll::Pending
    }
}

impl<S> Drop for PublisherDropWait<S> {
    fn drop(&mut self) {
        self.inner.lock().publisher_drop_waker = None;
    }
}

#[cfg(test)]
impl<S: Mergeable + Clone + Send + Sync + 'static> Context<S> {
    fn occupied_subscriber_wakers(&self, id: i32) -> usize {
        let Some(inner) = self.upgrade_inner(id) else {
            return 0;
        };
        inner
            .lock()
            .subscriber_wakers
            .iter()
            .filter(|waker| waker.is_some())
            .count()
    }

    fn halt_waker_is_some(&self, id: i32) -> bool {
        let Some(inner) = self.upgrade_inner(id) else {
            return false;
        };
        inner.lock().halt_waker.is_some()
    }

    fn publisher_drop_waker_is_some(&self, id: i32) -> bool {
        let Some(inner) = self.upgrade_inner(id) else {
            return false;
        };
        inner.lock().publisher_drop_waker.is_some()
    }

    fn inner_is_live(&self, id: i32) -> bool {
        self.upgrade_inner(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use tokio_stream::StreamExt;

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Chunk(String);

    impl Mergeable for Chunk {
        fn merge(&mut self, other: Self) -> Option<Self> {
            self.0.push_str(&other.0);
            None
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn slice(&self, range: Range<usize>) -> Option<Self> {
            self.0.get(range).map(|slice| Chunk(slice.to_string()))
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Piece {
        Text(String),
        Mark(u32),
    }

    impl Mergeable for Piece {
        fn merge(&mut self, other: Self) -> Option<Self> {
            match (self, other) {
                (Piece::Text(left), Piece::Text(right)) => {
                    left.push_str(&right);
                    None
                }
                (_, other) => Some(other),
            }
        }

        fn len(&self) -> usize {
            match self {
                Piece::Text(text) => text.len(),
                Piece::Mark(_) => 1,
            }
        }

        fn slice(&self, range: Range<usize>) -> Option<Self> {
            match self {
                Piece::Text(text) => text.get(range).map(|slice| Piece::Text(slice.to_string())),
                mark if range.start == 0 => Some(mark.clone()),
                _ => None,
            }
        }
    }

    async fn recv_timeout<S>(
        stream: &mut (impl Stream<Item = S> + Unpin),
        millis: u64,
    ) -> Option<S> {
        tokio::time::timeout(Duration::from_millis(millis), stream.next())
            .await
            .ok()
            .flatten()
    }

    fn concat_chunks(chunks: &[Chunk]) -> String {
        chunks.iter().map(|chunk| chunk.0.as_str()).collect()
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Chan {
        Think(String),
        Say(String),
    }

    impl Mergeable for Chan {
        fn merge(&mut self, other: Self) -> Option<Self> {
            match (self, other) {
                (Chan::Think(left), Chan::Think(right)) => {
                    left.push_str(&right);
                    None
                }
                (Chan::Say(left), Chan::Say(right)) => {
                    left.push_str(&right);
                    None
                }
                (_, other) => Some(other),
            }
        }

        fn len(&self) -> usize {
            match self {
                Chan::Think(text) | Chan::Say(text) => text.len(),
            }
        }

        fn slice(&self, range: Range<usize>) -> Option<Self> {
            match self {
                Chan::Think(text) => text.get(range).map(|slice| Chan::Think(slice.to_string())),
                Chan::Say(text) => text.get(range).map(|slice| Chan::Say(slice.to_string())),
            }
        }
    }

    async fn recv_all<S: Clone>(stream: &mut (impl Stream<Item = S> + Unpin)) -> Vec<S> {
        let mut items = Vec::new();
        while let Some(item) = recv_timeout(stream, 50).await {
            items.push(item);
        }
        items
    }

    #[tokio::test]
    async fn fast_pull_yields_deltas_without_leak_or_missing() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chunk("a".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("a".into()))
        );
        publisher.publish(Chunk("b".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("b".into()))
        );
        publisher.publish(Chunk("c".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("c".into()))
        );
        assert_eq!(recv_timeout(&mut stream, 30).await, None);
    }

    #[tokio::test]
    async fn slow_pull_merges_into_one_delta() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chunk("hel".into()));
        publisher.publish(Chunk("lo".into()));
        publisher.publish(Chunk("!".into()));

        let first = recv_timeout(&mut stream, 200).await.unwrap();
        assert_eq!(first, Chunk("hello!".into()));
        assert_eq!(recv_timeout(&mut stream, 30).await, None);
    }

    #[tokio::test]
    async fn throttle_mid_run_second_chunk_is_unread_tail() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chunk("hello".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("hello".into()))
        );

        publisher.publish(Chunk(" ".into()));
        publisher.publish(Chunk("world".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk(" world".into()))
        );
        assert_eq!(recv_timeout(&mut stream, 30).await, None);
    }

    #[tokio::test]
    async fn resume_from_cursor_returns_only_suffix() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chunk("hello".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("hello".into()))
        );
        drop(stream);

        let mut resumed = context.clone().subscribe(
            1,
            Some(Cursor {
                index: 0,
                offset: 5,
            }),
        );
        publisher.publish(Chunk(" world".into()));
        assert_eq!(
            recv_timeout(&mut resumed, 200).await,
            Some(Chunk(" world".into()))
        );
        assert_eq!(recv_timeout(&mut resumed, 30).await, None);
    }

    #[tokio::test]
    async fn interleaved_think_say_preserves_order_on_slow_pull() {
        let context = Arc::new(Context::<Chan>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chan::Think("a".into()));
        publisher.publish(Chan::Say("b".into()));
        publisher.publish(Chan::Think("c".into()));
        publisher.publish(Chan::Say("d".into()));

        let received = recv_all(&mut stream).await;
        assert_eq!(
            received,
            vec![
                Chan::Think("a".into()),
                Chan::Say("b".into()),
                Chan::Think("c".into()),
                Chan::Say("d".into()),
            ],
            "interleaved channels must stay abcd, not grouped acbd"
        );
    }

    #[tokio::test]
    async fn interleaved_think_say_preserves_order_on_fast_pull() {
        let context = Arc::new(Context::<Chan>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chan::Think("a".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chan::Think("a".into()))
        );
        publisher.publish(Chan::Say("b".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chan::Say("b".into()))
        );
        publisher.publish(Chan::Think("c".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chan::Think("c".into()))
        );
        publisher.publish(Chan::Say("d".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chan::Say("d".into()))
        );
    }

    #[tokio::test]
    async fn discrete_tokens_do_not_merge_with_text() {
        let context = Arc::new(Context::<Piece>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Piece::Text("ab".into()));
        publisher.publish(Piece::Text("cd".into()));
        publisher.publish(Piece::Mark(7));
        publisher.publish(Piece::Text("ef".into()));

        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Piece::Text("abcd".into()))
        );
        assert_eq!(recv_timeout(&mut stream, 200).await, Some(Piece::Mark(7)));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Piece::Text("ef".into()))
        );
    }

    #[tokio::test]
    async fn two_subscribers_both_reconstruct() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut first = context.clone().subscribe(1, None);
        let mut second = context.clone().subscribe(1, None);

        publisher.publish(Chunk("xy".into()));
        publisher.publish(Chunk("z".into()));

        assert_eq!(
            recv_timeout(&mut first, 200).await,
            Some(Chunk("xyz".into()))
        );
        assert_eq!(
            recv_timeout(&mut second, 200).await,
            Some(Chunk("xyz".into()))
        );
    }

    #[tokio::test]
    async fn publisher_drop_then_new_publish_does_not_mix_sessions() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut stream = context.clone().subscribe(1, None);

        {
            let mut publisher = context.clone().publish(1).unwrap();
            publisher.publish(Chunk("old".into()));
            assert_eq!(
                recv_timeout(&mut stream, 200).await,
                Some(Chunk("old".into()))
            );
        }

        assert!(context.publishable(1));

        let mut publisher = context.clone().publish(1).unwrap();
        publisher.publish(Chunk("new".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("new".into()))
        );
    }

    #[tokio::test]
    async fn stop_unblocks_wait_halt_without_in_flight_token() {
        let context = Arc::new(Context::<Chunk>::new());
        let publisher = context.clone().publish(1).unwrap();
        let halt = publisher.wait_halt();
        let stop_context = context.clone();
        let stop_task = tokio::spawn(async move { stop_context.stop(1).await });

        tokio::time::timeout(Duration::from_secs(1), halt)
            .await
            .expect("wait_halt must unblock");
        drop(publisher);
        tokio::time::timeout(Duration::from_secs(1), stop_task)
            .await
            .expect("stop must not hang")
            .expect("stop task");
    }

    #[tokio::test]
    async fn stop_without_publisher_does_not_hang() {
        let context = Arc::new(Context::<Chunk>::new());
        let _stream = context.clone().subscribe(1, None);
        tokio::time::timeout(Duration::from_secs(1), context.stop(1))
            .await
            .expect("stop must not hang when no publisher exists");
    }

    #[tokio::test]
    async fn empty_len_items_are_skipped() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        publisher.publish(Chunk(String::new()));
        publisher.publish(Chunk("ok".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("ok".into()))
        );
    }

    #[tokio::test]
    async fn subscriber_drop_clears_waker_slot() {
        let context = Arc::new(Context::<Chunk>::new());
        let _publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        assert_eq!(recv_timeout(&mut stream, 30).await, None);
        assert_eq!(context.occupied_subscriber_wakers(1), 1);

        drop(stream);
        assert_eq!(context.occupied_subscriber_wakers(1), 0);
    }

    #[tokio::test]
    async fn wait_halt_drop_clears_halt_waker() {
        let context = Arc::new(Context::<Chunk>::new());
        let publisher = context.clone().publish(1).unwrap();
        let mut halt = publisher.wait_halt();

        assert_eq!(
            tokio::time::timeout(Duration::from_millis(30), &mut halt)
                .await
                .ok(),
            None
        );
        assert!(context.halt_waker_is_some(1));

        drop(halt);
        assert!(!context.halt_waker_is_some(1));
    }

    #[tokio::test]
    async fn stop_future_drop_clears_publisher_drop_waker() {
        let context = Arc::new(Context::<Chunk>::new());
        let _publisher = context.clone().publish(1).unwrap();
        {
            let stop = context.stop(1);
            tokio::pin!(stop);
            assert_eq!(
                tokio::time::timeout(Duration::from_millis(30), stop.as_mut())
                    .await
                    .ok(),
                None
            );
            assert!(context.publisher_drop_waker_is_some(1));
        }
        assert!(!context.publisher_drop_waker_is_some(1));
    }

    #[tokio::test]
    async fn reconnect_does_not_accumulate_waker_slots() {
        let context = Arc::new(Context::<Chunk>::new());
        let _publisher = context.clone().publish(1).unwrap();

        for _ in 0..8 {
            let mut stream = context.clone().subscribe(1, None);
            assert_eq!(recv_timeout(&mut stream, 30).await, None);
            assert_eq!(context.occupied_subscriber_wakers(1), 1);
            drop(stream);
            assert_eq!(context.occupied_subscriber_wakers(1), 0);
        }
    }

    #[tokio::test]
    async fn never_streamed_client_disconnect_frees_inner() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut stream = context.clone().subscribe(1, None);
        assert_eq!(recv_timeout(&mut stream, 30).await, None);
        assert!(context.inner_is_live(1));
        drop(stream);
        assert!(
            !context.inner_is_live(1),
            "Inner must drop when the only subscriber disconnects without a publisher"
        );
    }

    #[tokio::test]
    async fn completed_stream_client_disconnect_frees_inner() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut stream = context.clone().subscribe(1, None);
        {
            let mut publisher = context.clone().publish(1).unwrap();
            publisher.publish(Chunk("done".into()));
            assert_eq!(
                recv_timeout(&mut stream, 200).await,
                Some(Chunk("done".into()))
            );
        }
        assert_eq!(recv_timeout(&mut stream, 30).await, None);
        assert!(context.inner_is_live(1));
        drop(stream);
        assert!(
            !context.inner_is_live(1),
            "Inner must drop after publisher closed and client disconnects"
        );
    }

    #[tokio::test]
    async fn invalid_cursor_does_not_abort_and_still_gets_new_data() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(
            1,
            Some(Cursor {
                index: 1_000_000,
                offset: 1_000_000,
            }),
        );

        publisher.publish(Chunk("old".into()));
        assert_eq!(recv_timeout(&mut stream, 30).await, None);

        publisher.publish(Chunk("new".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Chunk("new".into()))
        );
    }

    #[tokio::test]
    async fn invalid_offset_does_not_abort() {
        let context = Arc::new(Context::<Piece>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(
            1,
            Some(Cursor {
                index: 0,
                offset: 1_000_000,
            }),
        );
        publisher.publish(Piece::Mark(3));
        publisher.publish(Piece::Text("ok".into()));
        assert_eq!(
            recv_timeout(&mut stream, 200).await,
            Some(Piece::Text("ok".into()))
        );
    }

    #[tokio::test]
    async fn second_publisher_rejected_while_alive() {
        let context = Arc::new(Context::<Chunk>::new());
        let first = context.clone().publish(1).unwrap();
        assert!(!context.publishable(1));
        assert!(context.clone().publish(1).is_none());
        drop(first);
        assert!(context.publishable(1));
    }

    #[tokio::test]
    async fn published_concat_equals_received_concat() {
        let context = Arc::new(Context::<Chunk>::new());
        let mut publisher = context.clone().publish(1).unwrap();
        let mut stream = context.clone().subscribe(1, None);

        let published = ["The ", "quick ", "brown", " fox"];
        for part in published {
            publisher.publish(Chunk(part.into()));
        }

        let mut received = Vec::new();
        while let Some(chunk) = recv_timeout(&mut stream, 50).await {
            received.push(chunk);
        }
        assert_eq!(concat_chunks(&received), published.concat());
    }

    /// Exhaustive BFS of the mutex+waker protocol.
    ///
    /// Axioms: one mutex object; condition and waker register in the same CS;
    /// wake only after unlock; Drop of a waiter clears its slot.
    /// Invariants: no leak (cons ≤ prod), no missing at EOF, no lost wakeup,
    /// no waker uniquely owned after waiter is gone.
    #[test]
    fn model_check_protocol() {
        protocol_model::assert_holds();
    }

    mod protocol_model {
        use std::collections::{HashSet, VecDeque};

        const MAX_PROD: u8 = 2;

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum Phase {
            Idle,
            Poll,
            Pend,
            Done,
            Gone,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum HaltPhase {
            Idle,
            Poll,
            Pend,
            Done,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        enum MutexHeld {
            Free,
            Sub0,
            Sub1,
            Halt,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        struct S {
            mutex: MutexHeld,
            prod: u8,
            cons: [u8; 2],
            alive: bool,
            closed: bool,
            mapped: bool,
            halted: bool,
            phase: [Phase; 2],
            waker: [bool; 2],
            sched: [bool; 2],
            halt_phase: HaltPhase,
            waker_h: bool,
            sched_h: bool,
            dw: [bool; 3],
        }

        impl S {
            fn init() -> Self {
                Self::base(true)
            }

            fn init_never_streamed() -> Self {
                Self::base(false)
            }

            fn base(publisher_alive: bool) -> Self {
                Self {
                    mutex: MutexHeld::Free,
                    prod: 0,
                    cons: [0, 0],
                    alive: publisher_alive,
                    closed: false,
                    mapped: true,
                    halted: false,
                    phase: [Phase::Idle, Phase::Idle],
                    waker: [false, false],
                    sched: [true, true],
                    halt_phase: HaltPhase::Idle,
                    waker_h: false,
                    sched_h: false,
                    dw: [false, false, false],
                }
            }

            fn all_subscribers_gone(&self) -> bool {
                self.phase.iter().all(|phase| *phase == Phase::Gone)
            }
        }

        fn drain(state: S) -> Option<S> {
            if state.mutex == MutexHeld::Free && state.dw.iter().any(|flag| *flag) {
                let mut next = state;
                next.sched[0] |= state.dw[0];
                next.sched[1] |= state.dw[1];
                next.sched_h |= state.dw[2];
                next.dw = [false, false, false];
                return Some(next);
            }
            None
        }

        fn successors(state: S) -> Vec<S> {
            if let Some(next) = drain(state) {
                return vec![next];
            }
            let mut out = Vec::new();

            for index in 0..2 {
                if state.mutex == MutexHeld::Free
                    && state.sched[index]
                    && matches!(state.phase[index], Phase::Idle | Phase::Pend)
                {
                    let mut next = state;
                    next.mutex = if index == 0 {
                        MutexHeld::Sub0
                    } else {
                        MutexHeld::Sub1
                    };
                    next.phase[index] = Phase::Poll;
                    next.waker[index] = false;
                    next.sched[index] = false;
                    out.push(next);
                }
            }

            for index in 0..2 {
                let held = if index == 0 {
                    MutexHeld::Sub0
                } else {
                    MutexHeld::Sub1
                };
                if state.mutex != held || state.phase[index] != Phase::Poll {
                    continue;
                }
                if state.cons[index] < state.prod {
                    for reschedule in [false, true] {
                        let mut next = state;
                        next.mutex = MutexHeld::Free;
                        next.cons[index] = state.prod;
                        next.phase[index] = Phase::Idle;
                        next.sched[index] = reschedule;
                        out.push(next);
                    }
                } else if state.closed {
                    let mut next = state;
                    next.mutex = MutexHeld::Free;
                    next.phase[index] = Phase::Done;
                    next.waker[index] = false;
                    next.sched[index] = false;
                    out.push(next);
                } else {
                    let mut next = state;
                    next.mutex = MutexHeld::Free;
                    next.phase[index] = Phase::Pend;
                    next.waker[index] = true;
                    next.sched[index] = false;
                    out.push(next);
                }
            }

            if state.mutex == MutexHeld::Free && state.alive && state.prod < MAX_PROD {
                let mut next = state;
                next.prod += 1;
                let take0 = state.waker[0];
                let take1 = state.waker[1];
                next.waker = [false, false];
                next.dw[0] |= take0;
                next.dw[1] |= take1;
                out.push(next);
            }

            if state.mutex == MutexHeld::Free && state.alive {
                let mut next = state;
                next.alive = false;
                next.closed = true;
                next.mapped = false;
                let take0 = state.waker[0];
                let take1 = state.waker[1];
                next.waker = [false, false];
                next.dw[0] |= take0;
                next.dw[1] |= take1;
                out.push(next);
            }

            if state.mutex == MutexHeld::Free && !state.halted {
                let mut next = state;
                next.halted = true;
                let take = state.waker_h;
                next.waker_h = false;
                next.dw[2] |= take;
                out.push(next);
            }

            if state.mutex == MutexHeld::Free
                && state.halt_phase == HaltPhase::Idle
                && state.alive
                && !state.sched_h
                && !state.waker_h
            {
                let mut next = state;
                next.sched_h = true;
                out.push(next);
            }

            if state.mutex == MutexHeld::Free
                && state.sched_h
                && matches!(state.halt_phase, HaltPhase::Idle | HaltPhase::Pend)
            {
                let mut next = state;
                next.mutex = MutexHeld::Halt;
                next.halt_phase = HaltPhase::Poll;
                next.waker_h = false;
                next.sched_h = false;
                out.push(next);
            }

            if state.mutex == MutexHeld::Halt && state.halt_phase == HaltPhase::Poll {
                let mut next = state;
                next.mutex = MutexHeld::Free;
                if state.halted {
                    next.halt_phase = HaltPhase::Done;
                    next.waker_h = false;
                } else {
                    next.halt_phase = HaltPhase::Pend;
                    next.waker_h = true;
                }
                out.push(next);
            }

            for index in 0..2 {
                if state.mutex == MutexHeld::Free
                    && state.phase[index] == Phase::Idle
                    && !state.sched[index]
                {
                    let mut next = state;
                    next.sched[index] = true;
                    out.push(next);
                }
            }

            for index in 0..2 {
                if state.mutex == MutexHeld::Free
                    && matches!(state.phase[index], Phase::Idle | Phase::Pend)
                {
                    let mut next = state;
                    next.phase[index] = Phase::Gone;
                    next.waker[index] = false;
                    next.sched[index] = false;
                    if next.all_subscribers_gone() && !next.alive {
                        next.mapped = false;
                    }
                    out.push(next);
                }
            }

            if state.mutex == MutexHeld::Free
                && matches!(state.halt_phase, HaltPhase::Idle | HaltPhase::Pend)
            {
                let mut next = state;
                next.halt_phase = HaltPhase::Done;
                next.waker_h = false;
                next.sched_h = false;
                out.push(next);
            }

            out
        }

        fn violations(state: &S) -> Vec<&'static str> {
            let mut found = Vec::new();
            if state.cons[0] > state.prod || state.cons[1] > state.prod {
                found.push("leak");
            }
            for index in 0..2 {
                if state.phase[index] == Phase::Done && state.cons[index] != state.prod {
                    found.push("missing_eof");
                }
                if state.phase[index] == Phase::Pend
                    && state.mutex == MutexHeld::Free
                    && !state.waker[index]
                    && !state.sched[index]
                    && !state.dw[index]
                    && (state.cons[index] < state.prod || state.closed)
                {
                    found.push("lost_wakeup");
                }
                if state.phase[index] == Phase::Gone && state.waker[index] {
                    found.push("waker_leak");
                }
            }
            if state.all_subscribers_gone() && !state.alive && state.mapped {
                found.push("inner_leak");
            }
            if state.halt_phase == HaltPhase::Pend
                && state.mutex == MutexHeld::Free
                && !state.waker_h
                && !state.sched_h
                && !state.dw[2]
                && state.halted
            {
                found.push("lost_wakeup_halt");
            }
            if state.halt_phase == HaltPhase::Done && state.waker_h {
                found.push("halt_waker_leak");
            }
            found
        }

        fn explore(start: S) -> usize {
            let mut seen = HashSet::new();
            let mut queue = VecDeque::new();
            seen.insert(start);
            queue.push_back(start);

            while let Some(state) = queue.pop_front() {
                let found = violations(&state);
                assert!(found.is_empty(), "protocol violated {found:?} in {state:?}");
                for next in successors(state) {
                    if seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
            seen.len()
        }

        pub fn assert_holds() {
            let with_publisher = explore(S::init());
            let never_streamed = explore(S::init_never_streamed());
            assert!(
                with_publisher > 100 && never_streamed > 50,
                "model too small: publisher={with_publisher} never={never_streamed}"
            );
        }
    }
}
