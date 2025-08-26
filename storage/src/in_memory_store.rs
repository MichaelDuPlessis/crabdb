use crate::{Store, StoreResult};
use concurrent_map::ConcurrentMap;

/// Stores data in memory only
#[derive(Debug, Default)]
pub struct InMemoryStore {
    map: ConcurrentMap<object::Key, object::Object>,
}

impl InMemoryStore {
    /// Create a new InMemoryStore with a set number of shards
    pub fn new(num_shards: usize) -> Self {
        Self {
            map: ConcurrentMap::new(num_shards),
        }
    }
}

impl Store for InMemoryStore {
    fn store(&self, key: object::Key, object: object::Object) -> StoreResult {
        Ok(self.map.insert(key, object).into())
    }

    fn retrieve(&self, key: &object::Key) -> StoreResult {
        Ok(self.map.get(key).map(|object| object.clone()).into())
    }

    fn remove(&self, key: object::Key) -> StoreResult {
        Ok(self.map.remove(&key).into())
    }
}
