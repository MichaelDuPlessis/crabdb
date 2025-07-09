use concurrent_map::ConcurrentMap;
use object::{Key, Object};

/// This trait means that the type cna handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    /// otherwise return the Null Object
    fn store(&self, key: Key, object: Object) -> Object;

    /// Retrieve an Object from its Key if it exists otherwise return the Null Object
    fn retrieve(&self, key: Key) -> Object;

    /// Delete an Object from from its Key and return the deleted Object
    fn remove(&self, key: Key) -> Object;
}

/// Stores data in memory only
#[derive(Debug, Default)]
pub struct InMemoryStore {
    map: ConcurrentMap<Key, Object>,
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
