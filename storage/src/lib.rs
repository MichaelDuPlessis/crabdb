use object::{DbObject, Key};
use std::collections::HashMap;

/// This trait means that the type cna handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    fn store(&mut self, key: Key, object: DbObject) -> Option<DbObject>;

    /// Retrieve an Object from its Key
    fn retrieve(&self, key: Key) -> Option<DbObject>;
}

/// Stores data in memory only
#[derive(Debug)]
pub struct InMemoryStore {
    map: HashMap<Key, DbObject>,
}

impl Store for InMemoryStore {
    fn store(&mut self, key: Key, object: DbObject) -> Option<DbObject> {
        self.map.insert(key, object)
    }

    fn retrieve(&self, key: Key) -> Option<DbObject> {
        self.map.get(&key).map(|object| object.boxed_clone())
    }
}
