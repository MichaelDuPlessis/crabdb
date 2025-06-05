use crate::Storage;
use crab_core::{Key, Object};
use std::collections::HashMap;

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
    fn save(&mut self, key: Key, object: Object) -> crate::Result<()> {
        self.map.insert(key, object);
        Ok(())
    }

    fn retrieve(&self, key: impl AsRef<Key>) -> crate::Result<Option<&Object>> {
        let key = key.as_ref();
        Ok(self.map.get(key))
    }

    fn delete(&mut self, key: impl AsRef<Key>) -> crate::Result<Option<Object>> {
        let key = key.as_ref();
        Ok(self.map.remove(key))
    }
}
