use crate::{Store, StoredObject};
use concurrent_map::ConcurrentMap;
use object::{Key, Object};

/// Stores data in memory only
#[derive(Debug, Default)]
pub struct InMemoryStore {
    map: ConcurrentMap<Key, StoredObject>,
}

impl Store for InMemoryStore {
    fn store(&self, key: Key, object: Object) -> Object {
        let stored_object = StoredObject::new(object);
        self.map
            .insert(key, stored_object)
            .map(|old_stored| old_stored.object)
            .into()
    }

    fn retrieve(&self, key: Key) -> Object {
        self.map
            .get(&key)
            .map(|stored_object| stored_object.object.clone())
            .into()
    }

    fn remove(&self, key: Key) -> Object {
        self.map
            .remove(&key)
            .map(|stored_object| stored_object.object)
            .into()
    }

    fn get_updated_time(&self, key: Key) -> u64 {
        self.map
            .get(&key)
            .map(|stored_object| stored_object.updated_time)
            .unwrap_or(0)
    }
}
