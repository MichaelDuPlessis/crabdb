use object::{Key, Object};

pub mod append_only_log;
pub mod in_memory_store;

/// Used if storing data fails for some reason
pub struct StoreError;

pub type StoreResult = Result<Object, StoreError>;

/// This trait means that the type can handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    /// otherwise return the Null Object
    fn store(&self, key: Key, object: Object) -> StoreResult;

    /// Retrieve an Object from its Key if it exists otherwise return the Null Object
    fn retrieve(&self, key: Key) -> StoreResult;

    /// Delete an Object from from its Key and return the deleted Object
    fn remove(&self, key: Key) -> StoreResult;
}
