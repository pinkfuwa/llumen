use std::sync::Mutex;

use tokio::sync::{Notify, watch};

use super::mergeable::Mergeable;

/// Cursor tracks position in the buffer for reading.
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

/// Inner state for a channel.
pub struct Inner<S: Mergeable + Clone> {
    pub buffer: Mutex<Vec<S>>,
    pub sender: Mutex<Option<watch::Sender<()>>>,
    pub receiver: watch::Receiver<()>,
    pub stop_notify: Notify,
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

/// Advance the cursor through the buffer, returning the next item slice.
pub fn advance_cursor<S: Mergeable + Clone>(cursor: &mut Cursor, buffer: &[S]) -> Option<S> {
    let offset = cursor.offset;
    let index = cursor.index;

    let item = buffer.get(index)?;
    let len = item.len();

    if offset == len {
        if index + 1 < buffer.len() {
            cursor.index += 1;
            cursor.offset = 0;
            return advance_cursor(cursor, buffer);
        } else {
            return None;
        }
    }

    cursor.offset = len;

    match item.slice(offset..len) {
        Some(sliced) => Some(sliced),
        None => {
            if index + 1 < buffer.len() {
                cursor.index += 1;
                cursor.offset = 0;
                advance_cursor(cursor, buffer)
            } else {
                None
            }
        }
    }
}

/// Check if the cursor has exhausted the buffer.
pub fn check_buffer_exhausted<S: Mergeable + Clone>(cursor: &Cursor, buffer: &[S]) -> bool {
    cursor.index >= buffer.len()
        || (cursor.index == buffer.len() - 1 && cursor.offset >= buffer[cursor.index].len())
}
