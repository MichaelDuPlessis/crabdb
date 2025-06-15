use crate::Storage;
use crab_core::{DbObject, Key, factory::new_null_db_object};
use std::{borrow::Borrow, collections::HashMap};

/// A storage medium that exists only in memory and does not persist data
pub struct InMemoryStore {
    map: HashMap<Key, DbObject>,
}

impl InMemoryStore {
    /// Creates a new InMemoryStore
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for InMemoryStore {
    fn set(&mut self, key: Key, object: DbObject) -> crate::Result<DbObject> {
        Ok(self.map.insert(key, object).unwrap_or(new_null_db_object()))
    }

    fn get(&self, key: impl Borrow<Key>) -> crate::Result<DbObject> {
        let key = key.borrow();
        Ok(self.map.get(key).cloned().unwrap_or(new_null_db_object()))
    }

    fn delete(&mut self, key: impl Borrow<Key>) -> crate::Result<DbObject> {
        let key = key.borrow();
        Ok(self.map.remove(key).unwrap_or(new_null_db_object()))
    }
}
