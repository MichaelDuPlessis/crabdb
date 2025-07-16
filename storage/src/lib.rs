use concurrent_map::ConcurrentMap;
use object::{Key, Object};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a stored object with its timestamp
#[derive(Debug, Clone)]
struct StoredObject {
    object: Object,
    updated_time: u64, // Unix timestamp in seconds
}

impl StoredObject {
    fn new(object: Object) -> Self {
        let updated_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        Self {
            object,
            updated_time,
        }
    }
}

/// This trait means that the type can handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    /// otherwise return the Null Object
    fn store(&self, key: Key, object: Object) -> Object;

    /// Retrieve an Object from its Key if it exists otherwise return the Null Object
    fn retrieve(&self, key: Key) -> Object;

    /// Delete an Object from from its Key and return the deleted Object
    fn remove(&self, key: Key) -> Object;

    /// Get the updated_time for a key as a Unix timestamp in seconds
    /// Returns 0 if the key doesn't exist
    fn get_updated_time(&self, key: Key) -> u64;
}

/// Stores data in memory only
#[derive(Debug, Default)]
pub struct InMemoryStore {
    map: ConcurrentMap<Key, StoredObject>,
}

impl Store for InMemoryStore {
    fn store(&self, key: Key, object: Object) -> Object {
        let stored_object = StoredObject::new(object);
        self.map.insert(key, stored_object)
            .map(|old_stored| old_stored.object)
            .into()
    }

    fn retrieve(&self, key: Key) -> Object {
        self.map.get(&key)
            .map(|stored_object| stored_object.object.clone())
            .into()
    }

    fn remove(&self, key: Key) -> Object {
        self.map.remove(&key)
            .map(|stored_object| stored_object.object)
            .into()
    }

    fn get_updated_time(&self, key: Key) -> u64 {
        self.map.get(&key)
            .map(|stored_object| stored_object.updated_time)
            .unwrap_or(0)
    }
}
