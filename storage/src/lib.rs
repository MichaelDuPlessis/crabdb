use object::{Key, Object};
use std::{error::Error, fmt::Display};

pub mod append_only_log;
pub mod in_memory_store;

/// Used if storing data fails for some reason
#[derive(Debug)]
pub struct StoreError;

impl Error for StoreError {}

impl Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "An error was encountered while interacting with the store"
        )
    }
}

pub type StoreResult = Result<Object, StoreError>;

/// This trait means that the type can handle storing objects in the database
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    /// otherwise return the Null Object
    fn store(&self, key: Key, object: Object) -> StoreResult;

    /// Retrieve an Object from its Key if it exists otherwise return the Null Object
    fn retrieve(&self, key: &Key) -> StoreResult;

    /// Delete an Object from from its Key and return the deleted Object
    fn remove(&self, key: &Key) -> StoreResult;
}
