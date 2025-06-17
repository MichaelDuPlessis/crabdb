use crate::Storage;
use concurrent_map::ConcurrentMap;
use crab_core::{Key, factory::new_null_db_object, object::DbObject};
use std::borrow::Borrow;

/// A storage medium that exists only in memory and does not persist data
pub struct InMemoryStore {
    map: ConcurrentMap<Key, DbObject>,
}

impl InMemoryStore {
    /// Creates a new InMemoryStore
    pub fn new() -> Self {
        Self {
            map: ConcurrentMap::new(4),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStore {
    fn set(&self, key: Key, object: DbObject) -> crate::Result<DbObject> {
        Ok(self.map.insert(key, object).unwrap_or(new_null_db_object()))
    }

    fn get(&self, key: impl Borrow<Key>) -> crate::Result<DbObject> {
        let key = key.borrow();
        Ok(self.map.get_cloned(key).unwrap_or(new_null_db_object()))
    }

    fn delete(&self, key: impl Borrow<Key>) -> crate::Result<DbObject> {
        let key = key.borrow();
        Ok(self.map.remove(key).unwrap_or(new_null_db_object()))
    }
}
