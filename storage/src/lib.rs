use object::{Key, Object};
use std::collections::HashMap;

/// This trait means that the type cna handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    fn store(&mut self, key: Key, object: Object) -> Option<Object>;

    /// Retrieve an Object from its Key
    fn retrieve(&self, key: Key) -> Option<Object>;
}

/// Stores data in memory only
#[derive(Debug)]
pub struct InMemoryStore {
    map: HashMap<Key, Object>,
}

impl Store for InMemoryStore {
    fn store(&mut self, key: Key, object: Object) -> Option<Object> {
        self.map.insert(key, object)
    }

    fn retrieve(&self, key: Key) -> Option<Object> {
        self.map.get(&key).cloned()
    }
}
