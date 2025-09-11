use std::sync::Arc;

use redb::{Database, TableDefinition};
use std::sync::Mutex;

const MAX_CACHE_SIZE: usize = 8;

const TABLE: TableDefinition<i32, Vec<u8>> = TableDefinition::new("blobs");

pub struct BlobDB {
    inner: Database,
    cache: Mutex<[Option<(i32, Arc<Vec<u8>>)>; MAX_CACHE_SIZE]>,
}

impl BlobDB {
    pub fn new(inner: Database) -> Self {
        Self {
            inner,
            cache: Default::default(),
        }
    }

    async fn get_from_db(&self, id: i32) -> Option<Vec<u8>> {
        self.inner.begin_read().ok().and_then(|txn| {
            txn.open_table(TABLE).ok().and_then(|table| {
                table
                    .get(id)
                    .ok()
                    .flatten()
                    .and_then(|blob| Some(blob.value()))
            })
        })
    }

    fn put_cache(&self, id: i32, blob: Arc<Vec<u8>>) {
        let mut cache = self.cache.lock().unwrap();
        // if cache found, return
        if cache.iter().any(|x| x.as_ref().map(|y| y.0) == Some(id)) {
            return;
        }
        match cache.iter().position(Option::is_none) {
            Some(pos) => cache[pos] = Some((id, blob)),
            None => {
                let rand = fastrand::usize(0..MAX_CACHE_SIZE);
                cache[rand] = Some((id, blob));
            }
        }
    }

    fn find_from_cache(&self, id: i32) -> Option<Arc<Vec<u8>>> {
        let mut cache = self.cache.lock().unwrap();
        cache.iter().find_map(|x| match x {
            Some(x) if x.0 == id => Some(x.1.clone()),
            _ => None,
        })
    }

    fn delete_from_cache(&self, id: i32) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(pos) = cache.iter().position(|x| match x {
            Some(x) if x.0 == id => true,
            _ => false,
        }) {
            cache[pos] = None;
        }
    }

    pub async fn get(&self, id: i32) -> Option<Arc<Vec<u8>>> {
        if let Some(x) = self.find_from_cache(id) {
            return Some(x);
        }

        let data = self.get_from_db(id).await?;
        let data = Arc::new(data);

        self.put_cache(id, data.clone());

        Some(data)
    }

    pub fn insert(&self, id: i32, data: Vec<u8>) -> Result<(), redb::Error> {
        let txn = self.inner.begin_write()?;

        let data = Arc::new(data);
        {
            let mut table = txn.open_table(TABLE)?;
            table.insert(id, data.clone())?;
        }
        txn.commit()?;
        self.put_cache(id, data);
        Ok(())
    }

    pub fn delete(&self, id: i32) -> Result<(), redb::Error> {
        let txn = self.inner.begin_write()?;
        {
            let mut table = txn.open_table(TABLE)?;
            table.remove(id)?;
        }
        txn.commit()?;
        self.delete_from_cache(id);
        Ok(())
    }
}
