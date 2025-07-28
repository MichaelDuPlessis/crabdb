use crate::Store;
use object::Object;
use std::{fs::File, path::Path};

/// The operation that occured e.g. SET, DELETE
#[derive(Debug)]
enum Operation {
    /// Creates an object
    Set,
    /// Deletes an object
    Del,
}

/// The data that is persisted to the aol
struct Log {
    object: Object,
}

/// Stores data in an append only log but relies on a backing Store solution to retrieve data
#[derive(Debug)]
pub struct AppendOnlyLogStore<S: Store> {
    /// The file to write the operations too
    file: File,
    /// The backing Store that is also written and read to
    backing_store: S,
}

impl<S: Store> AppendOnlyLogStore<S> {
    /// Creates a new AppendOnlyLogStore using the backing store and a filepath
    pub fn new(path: impl AsRef<Path>, backing_store: S) -> Self {
        let file = File::options()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();

        Self {
            file,
            backing_store,
        }
    }
}

impl<S: Store> Store for AppendOnlyLogStore<S> {
    fn store(&self, key: object::Key, object: object::Object) -> object::Object {
        todo!()
    }

    fn retrieve(&self, key: object::Key) -> object::Object {
        self.backing_store.retrieve(key)
    }

    fn remove(&self, key: object::Key) -> object::Object {
        todo!()
    }
}
