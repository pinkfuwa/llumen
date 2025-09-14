use std::collections::VecDeque;

pub trait Mergeable
where
    Self: Sized,
{
    fn merge(self, other: Self) -> (Self, Option<Self>);
}

#[derive(Debug)]
pub struct Buffer<T: Mergeable> {
    queue: VecDeque<T>,
}

impl<T> Default for Buffer<T>
where
    T: Sized + Mergeable,
{
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<T: Mergeable> Buffer<T> {
    pub fn push(&mut self, token: T) {
        if let Some(last) = self.queue.pop_back() {
            let (merged, extra) = last.merge(token);
            self.queue.push_back(merged);
            if let Some(extra_tok) = extra {
                self.queue.push_back(extra_tok);
            }
        } else {
            self.queue.push_back(token);
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.queue.pop_front()
    }
}

impl<T: Mergeable, E> Mergeable for Result<T, E> {
    fn merge(self, other: Self) -> (Self, Option<Self>) {
        match (self, other) {
            (Ok(a), Ok(b)) => {
                let (merged, extra) = a.merge(b);
                (Ok(merged), extra.map(Ok))
            }
            (Err(e), _) => (Err(e), None),
            (_, Err(e)) => (Err(e), None),
        }
    }
}
