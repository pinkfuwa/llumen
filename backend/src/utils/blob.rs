use std::{path::Path, pin::Pin, task};

use bytes::Bytes;
use redb::*;
use tokio_stream::{Stream, StreamExt};

use std::sync::Arc;

pub const TABLE: TableDefinition<i32, &[u8]> = TableDefinition::new("blobs");

pub struct Reader {
    // use of 'static break redb
    guard: AccessGuard<'static, &'static [u8]>,
    txn: Option<ReadTransaction>,
}

impl Drop for Reader {
    fn drop(&mut self) {
        // IMPORTANT: guard is only safe to read when txn is open
        // this is likely a bug in redb that you can drop txn then use guard.
        self.txn.take().unwrap().close().ok();
    }
}

impl AsRef<[u8]> for Reader {
    fn as_ref(&self) -> &[u8] {
        self.guard.value()
    }
}

impl Reader {
    pub fn len(&self) -> usize {
        self.guard.value().len()
    }

    pub fn head(&self, size: usize) -> &[u8] {
        let end = std::cmp::min(size, self.len());
        &self.guard.value()[..end]
    }
}

pub struct BlobReader {
    reader: Arc<Reader>,
}

impl std::fmt::Debug for BlobReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobReader")
            .field("len", &self.reader.len())
            .finish()
    }
}

impl BlobReader {
    pub fn len(&self) -> usize {
        self.reader.len()
    }
}

impl AsRef<[u8]> for BlobReader {
    fn as_ref(&self) -> &[u8] {
        self.reader.as_ref().as_ref()
    }
}

impl Clone for BlobReader {
    fn clone(&self) -> Self {
        Self {
            reader: self.reader.clone(),
        }
    }
}

impl From<Reader> for BlobReader {
    fn from(reader: Reader) -> Self {
        Self {
            reader: Arc::new(reader),
        }
    }
}

const CHUNK_SIZE: usize = 256 * 1024;

pub struct MmapStream {
    reader: Arc<Reader>,
    position: usize,
}

impl MmapStream {
    fn new(reader: Reader) -> Self {
        Self {
            reader: Arc::new(reader),
            position: 0,
        }
    }
}

impl From<Reader> for MmapStream {
    fn from(reader: Reader) -> Self {
        Self::new(reader)
    }
}

impl Stream for MmapStream {
    type Item = Result<Bytes, axum::Error>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        if self.position >= self.reader.len() {
            return task::Poll::Ready(None);
        }

        let position = self.position;
        let end = std::cmp::min(position + CHUNK_SIZE, self.reader.len());
        let buf = Bytes::copy_from_slice(&self.reader.as_ref().as_ref()[position..end]);
        self.position = end;
        task::Poll::Ready(Some(Ok(buf)))
    }
}

#[derive(Clone)]
pub struct BlobDB {
    pub inner: Arc<Database>,
}

impl BlobDB {
    pub async fn new_from_path(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Arc::new(Database::create(path)?);
        Ok(Self::new(db))
    }

    pub fn new(inner: Arc<Database>) -> Self {
        Self { inner }
    }

    /// get Reader
    ///
    /// Please note redb use mmap, so it's blocking on page fault
    pub fn get(&self, id: i32) -> Option<Reader> {
        let txn = self.inner.begin_read().ok()?;
        let table = txn.open_table(TABLE).ok()?;

        let guard = table.get(id).ok()??;
        Some(Reader {
            guard,
            txn: Some(txn),
        })
    }

    /// read all data
    pub async fn get_vectored(&self, id: i32) -> Option<Vec<u8>> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || db.get(id).map(|reader| reader.as_ref().to_vec()))
            .await
            .ok()?
    }

    pub async fn insert_with_error<S, E>(
        &self,
        id: i32,
        size: usize,
        mut chunk_stream: S,
    ) -> Result<Result<(), E>, redb::Error>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin + Send,
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<bytes::Bytes>(1);

        let db = self.clone();
        let write_task = tokio::task::spawn_blocking(move || {
            let txn = db.inner.begin_write()?;

            {
                let mut table = txn.open_table(TABLE)?;
                let mut accessor = table.insert_reserve(id, size)?;
                let writer = accessor.as_mut();
                let mut wrote = 0;

                while let Some(chunk) = rx.blocking_recv() {
                    writer[wrote..wrote + chunk.len()].copy_from_slice(&chunk);
                    wrote += chunk.len();
                }
            }

            txn.commit()?;
            Ok::<(), redb::Error>(())
        });

        while let Some(chunk) = chunk_stream.next().await {
            match chunk {
                Err(e) => return Ok(Err(e)),
                Ok(b) => {
                    tx.send(b).await.map_err(|_| {
                        redb::Error::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "write task failed",
                        ))
                    })?;
                }
            }
        }

        drop(tx);

        write_task.await.map_err(|_| {
            redb::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "spawn_blocking failed",
            ))
        })??;

        Ok(Ok(()))
    }

    pub async fn insert<S>(&self, id: i32, size: usize, chunk_stream: S) -> Result<(), redb::Error>
    where
        S: Stream<Item = bytes::Bytes> + Unpin + 'static + Send,
    {
        self.insert_with_error(
            id,
            size,
            chunk_stream.map(Ok::<bytes::Bytes, std::convert::Infallible>),
        )
        .await?
        .unwrap();
        Ok(())
    }

    pub fn delete(&self, id: i32) -> Result<(), redb::Error> {
        let txn = self.inner.begin_write()?;
        {
            let mut table = txn.open_table(TABLE)?;
            table.remove(id)?;
        }
        txn.commit()?;
        Ok(())
    }
}
