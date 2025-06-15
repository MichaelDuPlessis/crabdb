use crate::Storage;
use crab_core::{Key, Object, null::Null};
use std::{borrow::Borrow, collections::HashMap};

/// A storage medium that exists only in memory and does not persist data
pub struct InMemoryStore {
    map: HashMap<Key, Box<dyn Object>>,
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
    fn set(&mut self, key: Key, object: Box<dyn Object>) -> crate::Result<Box<dyn Object>> {
        Ok(self.map.insert(key, object).unwrap_or(Box::new(Null)))
    }

    fn get(&self, key: impl Borrow<Key>) -> crate::Result<Box<dyn Object>> {
        let key = key.borrow();
        Ok(self.map.get(key).cloned().unwrap_or(Box::new(Null)))
    }

    fn delete(&mut self, key: impl Borrow<Key>) -> crate::Result<Box<dyn Object>> {
        let key = key.borrow();
        Ok(self.map.remove(key).unwrap_or(Box::new(Null)))
    }
}
