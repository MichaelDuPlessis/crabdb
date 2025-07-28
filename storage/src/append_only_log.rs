use crate::Store;
use std::{
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::Path,
    sync::Mutex,
};

/// The data type used to store the Log type
type LogType = u8;
/// The number of bytes the Log type requires
const LOG_TYPE_NUM_BYTES: usize = std::mem::size_of::<LogType>();

/// The data type used to store the number of bytes of an object
type LogSize = u64;
/// The number of bytes the Log size requires
const LOG_SIZE_NUM_BYTES: usize = std::mem::size_of::<LogSize>();

/// The Log that occured e.g. SET, DELETE
#[derive(Debug)]
enum Log {
    /// Creates an object
    /// The byte 1 represents Set
    Set(object::Key, object::Object),
    /// Deletes an object
    /// The byte 1 represents Del
    Del(object::Key),
}

impl Log {
    const SET: LogType = 0;
    const DEL: LogType = 1;

    /// Get a refernce to the key
    fn key(&self) -> &object::Key {
        match self {
            Log::Set(key, _) => key,
            Log::Del(key) => key,
        }
    }

    /// Log to a file
    fn append(&self, file: &mut File) {
        let mut data;

        match self {
            Log::Set(key, object) => {
                let key_bytes = key.to_bytes();
                let object_bytes = object.serialize();

                let log_size =
                    (key_bytes.len() + object_bytes.len() + LOG_TYPE_NUM_BYTES) as LogSize;

                data = Vec::with_capacity(log_size as usize + LOG_SIZE_NUM_BYTES);
                // adding log size
                data.extend_from_slice(&log_size.to_be_bytes());
                // adding operation
                data.push(Self::SET);
                // adding key
                data.extend_from_slice(&key_bytes);
                // adding object
                data.extend_from_slice(&object_bytes);
            }
            Log::Del(key) => {
                let key_bytes = key.to_bytes();

                let log_size = (key_bytes.len() + LOG_TYPE_NUM_BYTES) as LogSize;

                data = Vec::with_capacity(log_size as usize + LOG_SIZE_NUM_BYTES);
                // adding log size
                data.extend_from_slice(&log_size.to_be_bytes());
                // adding operation
                data.push(Self::DEL);
                // adding key
                data.extend_from_slice(&key_bytes);
            }
        }

        file.write_all(&data).unwrap();
    }
}

/// Stores data in an append only log but relies on a backing Store solution to retrieve data
#[derive(Debug)]
pub struct AppendOnlyLogStore<S: Store> {
    /// The log files
    files: Vec<Mutex<File>>, // Mutex instead of RwLock since this is only ever written to
    /// The backing Store that is also written and read to
    backing_store: S,
}

impl<S: Store> AppendOnlyLogStore<S> {
    /// Creates a new AppendOnlyLogStore using the backing store and a filepath to the directory storing the logs as well
    /// as the number of log files
    pub fn new(
        dir_path: impl AsRef<Path>,
        num_files: std::num::NonZeroUsize,
        backing_store: S,
    ) -> Self {
        let dir_path = dir_path.as_ref();
        let files = (0..num_files.get())
            .map(|i| {
                Mutex::new(
                    File::options()
                        .append(true)
                        .create(true)
                        .open(dir_path.join(i.to_string()))
                        .unwrap(),
                )
            })
            .collect();

        Self {
            files,
            backing_store,
        }
    }

    /// Appends a Log to the file
    fn log(&self, log: Log) {
        // first figure out what file must be written to
        let key = log.key();
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = hasher.finish() as usize % self.files.len();

        // get the file that needs to be written to
        let file = unsafe { self.files.get_unchecked(index) }; // I just modded by the len so it must be within range
        let mut file = file.lock().unwrap();

        log.append(&mut *file);
    }
}

impl<S: Store> Store for AppendOnlyLogStore<S> {
    fn store(&self, key: object::Key, object: object::Object) -> object::Object {
        // create log to store
        // TODO: get rid of this clone
        let log = Log::Set(key.clone(), object.clone());
        self.log(log);

        self.backing_store.store(key, object)
    }

    fn retrieve(&self, key: object::Key) -> object::Object {
        self.backing_store.retrieve(key)
    }

    fn remove(&self, key: object::Key) -> object::Object {
        // create log to delete
        // TODO: get rid of this clone
        let log = Log::Del(key.clone());
        self.log(log);

        self.backing_store.remove(key)
    }
}
