use crate::Storage;
use crab_core::{Key, Object};
use std::{borrow::Borrow, collections::HashMap};

/// A storage medium that exists only in memory and does not persist data
pub struct InMemoryStore {
    map: HashMap<Key, Object>,
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
    fn set(&mut self, key: Key, object: Object) -> crate::Result<Object> {
        Ok(self.map.insert(key, object).unwrap_or(Object::Null))
    }

    fn get(&self, key: impl Borrow<Key>) -> crate::Result<Object> {
        let key = key.borrow();
        Ok(self.map.get(key).cloned().unwrap_or(Object::Null))
    }

    fn delete(&mut self, key: impl Borrow<Key>) -> crate::Result<Object> {
        let key = key.borrow();
        Ok(self.map.remove(key).unwrap_or(Object::Null))
    }
}
