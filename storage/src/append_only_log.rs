use crate::{Store, StoreError, StoreResult};
use std::{
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Write},
    path::{Path, PathBuf},
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

/// Errors that can occur during AOL operations
#[derive(Debug)]
pub enum AolError {
    /// I/O error during file operations
    Io(std::io::Error),
    /// Error parsing objects during recovery
    ObjectParse(object::ObjectError),
    /// Corrupted log entry (invalid size or format)
    CorruptedEntry(String),
    /// Directory creation failed
    DirectoryCreation(std::io::Error),
    /// Error with the backing store
    BackingStore(StoreError),
}

impl std::fmt::Display for AolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AolError::Io(e) => write!(f, "I/O error: {}", e),
            AolError::ObjectParse(e) => write!(f, "Object parsing error: {:?}", e),
            AolError::CorruptedEntry(msg) => write!(f, "Corrupted log entry: {}", msg),
            AolError::DirectoryCreation(e) => write!(f, "Failed to create directory: {}", e),
            AolError::BackingStore(e) => write!(f, "The backing store encountered an error: {}", e),
        }
    }
}

impl std::error::Error for AolError {}

impl From<std::io::Error> for AolError {
    fn from(error: std::io::Error) -> Self {
        AolError::Io(error)
    }
}

impl From<object::ObjectError> for AolError {
    fn from(error: object::ObjectError) -> Self {
        AolError::ObjectParse(error)
    }
}

impl From<AolError> for StoreError {
    fn from(_: AolError) -> Self {
        StoreError
    }
}

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

    /// Get a reference to the key
    fn key(&self) -> &object::Key {
        match self {
            Log::Set(key, _) => key,
            Log::Del(key) => key,
        }
    }

    /// Log to a file with proper error handling and synchronization
    fn append(&self, file: &mut File) -> Result<(), AolError> {
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

        // Write data and sync to disk for durability
        file.write_all(&data)?;
        file.sync_all()?;
        Ok(())
    }

    /// Parse a log entry from bytes
    fn from_bytes(mut data: &[u8]) -> Result<Self, AolError> {
        if data.len() < LOG_TYPE_NUM_BYTES {
            return Err(AolError::CorruptedEntry(
                "Not enough data for log type".to_string(),
            ));
        }

        let log_type = data[0];
        data = &data[1..];

        match log_type {
            Self::SET => {
                // Parse key
                let (key, remaining) = object::Key::new(data)?;
                // Parse object
                let (object, _) = object::Object::deserialize(remaining)?;
                Ok(Log::Set(key, object))
            }
            Self::DEL => {
                // Parse key
                let (key, _) = object::Key::new(data)?;
                Ok(Log::Del(key))
            }
            _ => Err(AolError::CorruptedEntry(format!(
                "Unknown log type: {}",
                log_type
            ))),
        }
    }
}

/// Stores data in an append only log but relies on a backing Store solution to retrieve data
#[derive(Debug)]
pub struct AppendOnlyLogStore<S: Store> {
    /// The log files
    files: Vec<Mutex<File>>, // Mutex instead of RwLock since this is only ever written to
    /// Directory path for recovery
    dir_path: PathBuf,
    /// The backing Store that is also written and read to
    backing_store: S,
}

impl<S: Store> AppendOnlyLogStore<S> {
    /// Creates a new AppendOnlyLogStore using the backing store and a filepath to the directory storing the logs
    /// This version does NOT perform recovery - use new_with_recovery for that
    pub fn new(
        dir_path: impl AsRef<Path>,
        num_files: std::num::NonZeroUsize,
        backing_store: S,
    ) -> Result<Self, AolError> {
        let dir_path = dir_path.as_ref();

        // Create directory if it doesn't exist
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path).map_err(AolError::DirectoryCreation)?;
        }

        let mut files = Vec::with_capacity(num_files.get());

        for i in 0..num_files.get() {
            let file_path = dir_path.join(i.to_string());
            let file = File::options()
                .append(true)
                .create(true)
                .read(true)
                .open(&file_path)?;
            files.push(Mutex::new(file));
        }

        Ok(Self {
            files,
            dir_path: dir_path.to_path_buf(),
            backing_store,
        })
    }

    /// Creates a new AppendOnlyLogStore and performs recovery from existing log files
    pub fn new_with_recovery(
        dir_path: impl AsRef<Path>,
        num_files: std::num::NonZeroUsize,
        backing_store: S,
    ) -> Result<Self, AolError> {
        // Create the store first
        let mut store = Self::new(dir_path, num_files, backing_store)?;

        // Perform recovery
        store.recover()?;

        Ok(store)
    }

    /// Recover data from log files by replaying all operations
    fn recover(&mut self) -> Result<(), AolError> {
        for i in 0..self.files.len() {
            self.recover_from_file_index(i)?;
        }
        Ok(())
    }

    /// Recover from a single log file using its index
    fn recover_from_file_index(&mut self, file_index: usize) -> Result<(), AolError> {
        let file_mutex = &self.files[file_index];
        let mut file = file_mutex.lock().map_err(|_| {
            AolError::CorruptedEntry("Failed to acquire file lock for recovery".to_string())
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut offset = 0;
        while offset < buffer.len() {
            // Check if we have enough bytes for the log size
            if offset + LOG_SIZE_NUM_BYTES > buffer.len() {
                // Incomplete entry at end of file - this can happen if the process crashed
                // during a write. We'll ignore it.
                break;
            }

            // Read log size
            let log_size_bytes = &buffer[offset..offset + LOG_SIZE_NUM_BYTES];
            let log_size = u64::from_be_bytes(
                log_size_bytes
                    .try_into()
                    .map_err(|_| AolError::CorruptedEntry("Invalid log size bytes".to_string()))?,
            ) as usize;

            offset += LOG_SIZE_NUM_BYTES;

            // Check if we have enough bytes for the log entry
            if offset + log_size > buffer.len() {
                // Incomplete entry - ignore and stop recovery
                break;
            }

            // Parse and apply the log entry
            let log_data = &buffer[offset..offset + log_size];
            let log_entry = Log::from_bytes(log_data)?;

            // Apply the operation to the backing store
            if let Err(e) = match log_entry {
                Log::Set(key, object) => self.backing_store.store(key, object),
                Log::Del(key) => self.backing_store.remove(key),
            } {
                return Err(AolError::BackingStore(e));
            }

            offset += log_size;
        }

        Ok(())
    }

    /// Appends a Log to the appropriate file
    fn log(&self, log: Log) -> Result<(), AolError> {
        // Figure out what file must be written to
        let key = log.key();
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let index = hasher.finish() as usize % self.files.len();

        // Get the file that needs to be written to
        let file_mutex = &self.files[index]; // Safe indexing - we just modded by len
        let mut file = file_mutex
            .lock()
            .map_err(|_| AolError::CorruptedEntry("Failed to acquire file lock".to_string()))?;

        log.append(&mut *file)
    }
}

impl<S: Store> Store for AppendOnlyLogStore<S> {
    fn store(&self, key: object::Key, object: object::Object) -> StoreResult {
        // create log to store
        // TODO: get rid of this clone
        let log = Log::Set(key.clone(), object.clone());
        if let Err(e) = self.log(log) {
            // If AOL write fails, we should probably fail the entire operation
            // For now, we'll log the error but continue (you might want to change this)
            eprintln!("Warning: Failed to write to AOL: {}", e);
            Err(e.into())
        } else {
            self.backing_store.store(key, object)
        }
    }

    fn retrieve(&self, key: object::Key) -> StoreResult {
        self.backing_store.retrieve(key)
    }

    fn remove(&self, key: object::Key) -> StoreResult {
        // create log to delete
        // TODO: get rid of this clone
        let log = Log::Del(key.clone());
        if let Err(e) = self.log(log) {
            eprintln!("Warning: Failed to write to AOL: {}", e);
            Err(e.into())
        } else {
            self.backing_store.remove(key)
        }
    }
}
