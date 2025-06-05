use crab_core::{Key, Object};

mod in_memory_store;

/// The types of errors that can occur with a storage medium
pub enum StorageError {}

type Result<T> = std::result::Result<T, StorageError>;

/// A common interface for storing items
trait Storage {
    /// Saves an object in the database under a Key and returns the old object under the key if there is one
    fn save(&mut self, key: Key, object: Object) -> Result<Option<Object>>;

    /// Gets an object saved in the database under a key or none if the key is not found
    fn retrieve(&self, key: impl AsRef<Key>) -> Result<Option<&Object>>;

    /// Deletes an object from the database under a key and returns the object or none if the key is not found
    fn delete(&mut self, key: impl AsRef<Key>) -> Result<Option<Object>>;
}
