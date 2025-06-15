use crab_core::{DbObject, Key};
use std::borrow::Borrow;

pub mod in_memory_store;

/// The types of errors that can occur with a storage medium
#[derive(Debug)]
pub enum StorageError {
    /// The value failed to be set
    SetFailed,
}

type Result<T> = std::result::Result<T, StorageError>;

/// A common interface for storing items
pub trait Storage {
    /// Saves an object in the database under a Key and returns the old object under the key if there is one
    fn set(&mut self, key: Key, object: DbObject) -> Result<DbObject>;

    /// Gets an object saved in the database under a key or none if the key is not found
    fn get(&self, key: impl Borrow<Key>) -> Result<DbObject>;

    /// Deletes an object from the database under a key and returns the object or none if the key is not found
    fn delete(&mut self, key: impl Borrow<Key>) -> Result<DbObject>;
}
