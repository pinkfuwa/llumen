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

    #[allow(deprecated)]
    pub async fn insert_with_error<S, E>(
        &self,
        id: i32,
        size: usize,
        chunk_stream: S,
    ) -> Result<Result<(), E>, redb::Error>
    where
        S: Stream<Item = Result<bytes::Bytes, E>> + Send,
    {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<bytes::Bytes>(1);
        let (commit_tx, commit_rx) = tokio::sync::oneshot::channel::<()>();

        let db = self.clone();
        let write_task = tokio::task::spawn_blocking(move || {
            let txn = db.inner.begin_write()?;

            {
                let mut table = txn.open_table(TABLE)?;
                let mut accessor = table.insert_reserve(id, size)?;
                let writer = accessor.as_mut();
                let mut wrote = 0;

                while let Some(chunk) = rx.blocking_recv() {
                    if chunk.len() > size - wrote {
                        return Err(redb::Error::Io(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "upload exceeds declared size",
                        )));
                    }
                    writer[wrote..wrote + chunk.len()].copy_from_slice(&chunk);
                    wrote += chunk.len();
                }

                if commit_rx.blocking_recv().is_err() {
                    return Ok(());
                }
                if wrote != size {
                    return Err(redb::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "upload is shorter than declared size",
                    )));
                }
            }

            txn.commit()?;
            Ok::<(), redb::Error>(())
        });

        let mut stream_error = None;
        let mut writer_closed = false;
        tokio::pin!(chunk_stream);
        while let Some(chunk) = chunk_stream.next().await {
            match chunk {
                Err(e) => {
                    stream_error = Some(e);
                    break;
                }
                Ok(b) => {
                    if tx.send(b).await.is_err() {
                        writer_closed = true;
                        break;
                    }
                }
            }
        }

        drop(tx);
        if stream_error.is_none() && !writer_closed {
            // Cancellation or a failed stream must not commit a partial upload.
            if commit_tx.send(()).is_err() {
                log::debug!("blob writer stopped before upload completed");
            }
        } else {
            drop(commit_tx);
        }

        write_task.await.map_err(|_| {
            redb::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "spawn_blocking failed",
            ))
        })??;

        Ok(match stream_error {
            Some(error) => Err(error),
            None => Ok(()),
        })
    }

    pub async fn insert<S>(&self, id: i32, size: usize, chunk_stream: S) -> Result<(), redb::Error>
    where
        S: Stream<Item = bytes::Bytes> + 'static + Send,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> anyhow::Result<BlobDB> {
        let database = Database::builder().create_with_backend(backends::InMemoryBackend::new())?;
        Ok(BlobDB::new(Arc::new(database)))
    }

    #[tokio::test]
    async fn stores_multiple_chunks_and_bounds_reader_head() -> anyhow::Result<()> {
        let database = database()?;
        assert!(database.get(1).is_none());
        database
            .insert(
                1,
                6,
                tokio_stream::iter([
                    Bytes::from_static(b"ab"),
                    Bytes::new(),
                    Bytes::from_static(b"cdef"),
                ]),
            )
            .await?;
        let reader = database
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("missing blob"))?;
        assert_eq!(reader.len(), 6);
        assert_eq!(reader.head(0), b"");
        assert_eq!(reader.head(3), b"abc");
        assert_eq!(reader.head(usize::MAX), b"abcdef");
        assert_eq!(
            database.get_vectored(1).await.as_deref(),
            Some(b"abcdef".as_slice())
        );
        Ok(())
    }

    #[tokio::test]
    async fn download_stream_preserves_bytes_at_chunk_boundaries() -> anyhow::Result<()> {
        for size in [
            0,
            1,
            CHUNK_SIZE - 1,
            CHUNK_SIZE,
            CHUNK_SIZE + 1,
            CHUNK_SIZE * 2 + 7,
        ] {
            let database = database()?;
            let content: Vec<u8> = (0..size).map(|index| (index % 251) as u8).collect();
            database
                .insert(1, size, tokio_stream::iter([Bytes::from(content.clone())]))
                .await?;
            let reader = database
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("missing blob"))?;
            let mut stream = MmapStream::from(reader);
            let mut received = Vec::new();
            let mut lengths = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                lengths.push(chunk.len());
                received.extend_from_slice(&chunk);
            }
            assert_eq!(received, content, "size {size}");
            assert_eq!(lengths.len(), size.div_ceil(CHUNK_SIZE));
            assert!(
                lengths
                    .iter()
                    .all(|length| *length > 0 && *length <= CHUNK_SIZE)
            );
            assert!(stream.next().await.is_none());
        }
        Ok(())
    }

    #[tokio::test]
    async fn reader_keeps_its_snapshot_after_replacement_and_deletion() -> anyhow::Result<()> {
        let database = database()?;
        database
            .insert(1, 3, tokio_stream::iter([Bytes::from_static(b"old")]))
            .await?;
        let reader = database
            .get(1)
            .ok_or_else(|| anyhow::anyhow!("missing blob"))?;
        let reader = BlobReader::from(reader);
        let cloned = reader.clone();
        database
            .insert(1, 3, tokio_stream::iter([Bytes::from_static(b"new")]))
            .await?;
        assert_eq!(
            database.get_vectored(1).await.as_deref(),
            Some(b"new".as_slice())
        );
        database.delete(1)?;
        database.delete(1)?;
        assert!(database.get(1).is_none());
        drop(reader);
        assert_eq!(cloned.len(), 3);
        assert_eq!(cloned.as_ref(), b"old");
        Ok(())
    }

    #[tokio::test]
    async fn failed_upload_rolls_back_and_preserves_the_original_blob() -> anyhow::Result<()> {
        let database = database()?;
        database
            .insert(1, 3, tokio_stream::iter([Bytes::from_static(b"old")]))
            .await?;
        for id in [1, 2] {
            let result = database
                .insert_with_error(
                    id,
                    6,
                    tokio_stream::iter([
                        Ok(Bytes::from_static(b"part")),
                        Err("upload interrupted"),
                    ]),
                )
                .await?;
            assert_eq!(result, Err("upload interrupted"));
        }
        assert_eq!(
            database.get_vectored(1).await.as_deref(),
            Some(b"old".as_slice())
        );
        assert!(database.get(2).is_none());
        Ok(())
    }

    #[tokio::test]
    async fn incorrect_upload_sizes_roll_back_without_panicking() -> anyhow::Result<()> {
        let database = database()?;
        database
            .insert(1, 3, tokio_stream::iter([Bytes::from_static(b"old")]))
            .await?;
        for size in [2, 4] {
            for id in [1, 2] {
                let error = database
                    .insert(id, size, tokio_stream::iter([Bytes::from_static(b"abc")]))
                    .await
                    .expect_err("incorrect upload size must be rejected");
                match error {
                    redb::Error::Io(error) => {
                        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
                    }
                    error => panic!("unexpected storage error: {error}"),
                }
                assert_eq!(
                    database.get_vectored(1).await.as_deref(),
                    Some(b"old".as_slice())
                );
                assert!(database.get(2).is_none());
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn cancelled_upload_does_not_commit_a_partial_blob() -> anyhow::Result<()> {
        let database = database()?;
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let upload = tokio::spawn({
            let database = database.clone();
            async move {
                let chunks = tokio_stream::iter([
                    Ok::<_, std::convert::Infallible>(Bytes::from_static(b"p")),
                    Ok(Bytes::from_static(b"a")),
                    Ok(Bytes::from_static(b"r")),
                ])
                .chain(futures_util::stream::once(async move {
                    started_tx.send(()).expect("test receiver must be open");
                    std::future::pending().await
                }));
                database.insert_with_error(1, 8, chunks).await
            }
        });
        started_rx.await?;
        upload.abort();
        assert!(
            upload
                .await
                .expect_err("upload should be cancelled")
                .is_cancelled()
        );
        let inner = database.inner.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            inner.begin_write()?.abort()?;
            Ok(())
        })
        .await??;
        assert!(database.get(1).is_none());
        Ok(())
    }
}
