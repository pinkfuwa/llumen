use std::path::Path;

use redb::*;
use tokio_stream::{Stream, StreamExt};

const TABLE: TableDefinition<i32, &[u8]> = TableDefinition::new("blobs");

struct Reader {
    accessor: AccessGuard<'static, &'static [u8]>,
    read: usize,
}

const MAX_CHUNK_LEN: usize = 512 * 1024;

impl Stream for Reader {
    type Item = bytes::Bytes;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.read >= self.accessor.value().len() {
            std::task::Poll::Ready(None)
        } else {
            let chunk_len = std::cmp::min(MAX_CHUNK_LEN, self.accessor.value().len() - self.read);
            std::task::Poll::Ready(Some(bytes::Bytes::from(
                self.accessor.value()[self.read..self.read + chunk_len].to_vec(),
            )))
        }
    }
}

pub struct BlobDB {
    inner: Database,
}

impl BlobDB {
    pub async fn new_from_path(path: impl AsRef<Path>) -> Result<Self, redb::Error> {
        let db = Database::create(path)?;
        Ok(Self::new(db))
    }

    pub fn new(inner: Database) -> Self {
        Self { inner }
    }

    pub async fn get(&self, id: i32) -> Option<Vec<u8>> {
        let txn = self.inner.begin_read().ok()?;
        let table = txn.open_table(TABLE).ok()?;

        let data = table.get(id).ok()??;
        Some(data.value().to_vec())
    }

    pub async fn get_reader(&self, id: i32) -> Option<impl Stream<Item = bytes::Bytes>> {
        let txn = self.inner.begin_read().ok()?;
        let table = txn.open_table(TABLE).ok()?;

        let data = table.get(id).ok()??;
        Some(Reader {
            accessor: data,
            read: 0,
        })
    }

    pub async fn insert_with_error<S, E>(
        &self,
        id: i32,
        size: usize,
        mut chunk_stream: S,
    ) -> Result<Result<(), E>, redb::Error>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Unpin,
    {
        let txn = self.inner.begin_write()?;

        {
            let mut table = txn.open_table(TABLE)?;
            let mut accessor = table.insert_reserve(id, size)?;

            let writer = accessor.as_mut();
            let mut wrote = 0;

            while let Some(chunk) = chunk_stream.next().await {
                match chunk {
                    Err(e) => return Ok(Err(e)),
                    Ok(b) => {
                        writer[wrote..wrote + b.len()].copy_from_slice(&b);
                        wrote += b.len();
                    }
                }
            }
        }

        txn.commit()?;

        Ok(Ok(()))
    }
    pub async fn insert<S>(&self, id: i32, size: usize, chunk_stream: S) -> Result<(), redb::Error>
    where
        S: Stream<Item = bytes::Bytes> + Unpin + 'static,
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
