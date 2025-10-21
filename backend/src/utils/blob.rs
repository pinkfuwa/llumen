use std::io::Read;
use std::path::Path;

use redb::*;
use tokio_stream::{Stream, StreamExt};

use std::sync::Arc;

pub const TABLE: TableDefinition<i32, &[u8]> = TableDefinition::new("blobs");

pub const MAX_CHUNK_LEN: usize = 512 * 1024;

pub struct Reader(AccessGuard<'static, &'static [u8]>);

impl AsRef<[u8]> for Reader {
    fn as_ref(&self) -> &[u8] {
        self.0.value()
    }
}

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

    pub async fn get(&self, id: i32) -> Option<Reader> {
        let txn = self.inner.begin_read().ok()?;
        let table = txn.open_table(TABLE).ok()?;

        let guard = table.get(id).ok()??;
        Some(Reader(guard))
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
        let txn = (&*self.inner).begin_write()?;

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
