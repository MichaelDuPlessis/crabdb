use crate::Store;
use concurrent_map::ConcurrentMap;
use object::{Key, Object};

/// Stores data in memory only
#[derive(Debug, Default)]
pub struct InMemoryStore {
    map: ConcurrentMap<Key, Object>,
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
    fn store(&self, key: Key, object: Object) -> Object {
        self.map.insert(key, object).into()
    }

    fn retrieve(&self, key: Key) -> Object {
        self.map.get(&key).map(|object| object.clone()).into()
    }

    fn remove(&self, key: Key) -> Object {
        self.map.remove(&key).into()
    }
}
