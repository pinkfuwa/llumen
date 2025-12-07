use std::path::Path;

use redb::*;
use tokio_stream::{Stream, StreamExt};

use std::sync::Arc;

pub const TABLE: TableDefinition<i32, &[u8]> = TableDefinition::new("blobs");

pub struct Reader {
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
