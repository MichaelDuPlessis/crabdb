# CrabDB Storage Layer

This module provides the storage backends for CrabDB, implementing the `Store` trait with both in-memory and persistent storage options.

## Store Trait

All storage backends implement the `Store` trait:

```rust
pub trait Store {
    /// Store an Object on a Key. If an object is already stored on that Key return it
    /// otherwise return the Null Object
    fn store(&self, key: Key, object: Object) -> Object;

    /// Retrieve an Object from its Key if it exists otherwise return the Null Object
    fn retrieve(&self, key: Key) -> Object;

    /// Delete an Object from from its Key and return the deleted Object
    fn remove(&self, key: Key) -> Object;
}
```

## Storage Backends

### InMemoryStore

A high-performance in-memory storage backend using CrabDB's custom concurrent hash map.

#### Features
- **Thread-Safe**: Uses `ConcurrentMap` for safe concurrent access
- **Configurable Sharding**: Adjustable number of internal shards for performance tuning
- **Zero Persistence**: Data is lost when the process terminates
- **High Performance**: Direct memory access with minimal overhead

#### Usage
```rust
use storage::InMemoryStore;

// Create with default sharding
let store = InMemoryStore::default();

// Create with custom shard count for better concurrency
let store = InMemoryStore::new(8);  // 8 internal shards
```

#### Implementation Details
- **Backing Structure**: `ConcurrentMap<Key, Object>`
- **Concurrency**: Lock-free reads, minimal write contention
- **Memory Usage**: Stores objects directly in memory
- **Performance**: O(1) average case for all operations

#### When to Use
- Development and testing
- Caching layer
- High-performance scenarios where persistence isn't required
- As a backing store for persistent layers (like AOL)

---

### AppendOnlyLogStore (AOL)

A persistent storage layer that wraps any `Store` implementation with durability guarantees using an append-only log approach.

#### Architecture

The AOL uses a decorator pattern, providing persistence for any backing store:

```text
┌─────────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Client Request    │───▶│  AppendOnlyLog   │───▶│  Backing Store  │
│  (SET/GET/DELETE)   │    │   (Persistence)  │    │  (In-Memory)    │
└─────────────────────┘    └──────────────────┘    └─────────────────┘
                                     │
                                     ▼
                           ┌──────────────────┐
                           │   Log Files      │
                           │  (0, 1, 2, 3)    │
                           └──────────────────┘
```

#### File Format

Each log entry follows this binary format:

```text
┌─────────────┬─────────────┬─────────────┬─────────────┐
│  Log Size   │ Operation   │    Key      │   Object    │
│  (8 bytes)  │  (1 byte)   │ (variable)  │ (variable)  │
└─────────────┴─────────────┴─────────────┴─────────────┘
```

**Field Details:**
- **Log Size**: Total size of operation + key + object data (big-endian u64)
- **Operation**: 
  - `0` = SET operation
  - `1` = DELETE operation
- **Key**: Serialized key with CrabDB's key format (length prefix + data)
- **Object**: Serialized object (only present for SET operations)

#### File Sharding

Operations are distributed across multiple log files using key hashing:

- **Purpose**: Reduces lock contention between concurrent operations
- **Benefit**: Enables parallel recovery during startup
- **Naming**: Files are named simply as numbers: `0`, `1`, `2`, `3`, etc.
- **Distribution**: `hash(key) % num_files` determines which file to use

#### Durability Guarantees

1. **Write-Ahead Logging**
   - Log entries are written to disk **before** updating the in-memory store
   - Ensures data survives crashes even if memory updates are lost

2. **Synchronous I/O**
   - Each write is followed by `fsync()` to force data to disk
   - Prevents data loss from OS buffer crashes

3. **Crash Recovery**
   - On startup, all log files are replayed to restore complete state
   - Operations are applied to the backing store in chronological order

4. **Partial Write Handling**
   - Incomplete entries (from crashes during writes) are safely ignored
   - Recovery stops at the first incomplete entry per file

#### Usage Examples

**Basic Usage (No Recovery):**
```rust
use storage::{AppendOnlyLogStore, InMemoryStore};

// Create AOL without recovery (for new databases)
let store = AppendOnlyLogStore::new(
    "./data",                                    // Directory path
    std::num::NonZeroUsize::new(4).unwrap(),    // Number of log files
    InMemoryStore::new(4)                       // Backing store
)?;
```

**With Recovery:**
```rust
// Create AOL with automatic recovery (for existing databases)
let store = AppendOnlyLogStore::new_with_recovery(
    "./data",                                    // Directory path
    std::num::NonZeroUsize::new(4).unwrap(),    // Number of log files
    InMemoryStore::new(4)                       // Backing store
)?;
```

**Integration with Engine:**
```rust
// In engine/src/main.rs
let storage = match AppendOnlyLogStore::new_with_recovery(
    "./data",
    unsafe { std::num::NonZeroUsize::new_unchecked(4) },
    InMemoryStore::new(4),
) {
    Ok(store) => Arc::new(store),
    Err(e) => {
        error!("Failed to initialize storage: {}", e);
        return;
    }
};
```

#### Error Handling

The AOL implementation provides comprehensive error handling through the `AolError` enum:

- **`Io(std::io::Error)`**: File I/O errors
- **`ObjectParse(object::ObjectError)`**: Object deserialization errors during recovery
- **`CorruptedEntry(String)`**: Malformed log entries
- **`DirectoryCreation(std::io::Error)`**: Directory creation failures

#### Recovery Process

1. **Startup**: `new_with_recovery()` is called
2. **File Reading**: Each log file is read entirely into memory
3. **Entry Parsing**: Log entries are parsed sequentially
4. **Operation Replay**: Each valid operation is applied to the backing store
5. **Incomplete Handling**: Partial entries at file end are ignored
6. **Ready**: Store is ready for new operations

#### Performance Characteristics

**Write Performance:**
- **Overhead**: Each write operation requires disk I/O + fsync
- **Concurrency**: Multiple files reduce lock contention
- **Bottleneck**: Disk write speed and fsync latency

**Read Performance:**
- **Speed**: Reads are served directly from the in-memory backing store
- **No Overhead**: AOL doesn't impact read performance

**Recovery Performance:**
- **Startup Time**: Proportional to total log file size
- **Memory Usage**: Temporary spike during recovery (file contents loaded)
- **Parallelization**: Could be improved with concurrent file processing

#### File Management

**Directory Structure:**
```
./data/
├── 0          # Log file 0
├── 1          # Log file 1
├── 2          # Log file 2
└── 3          # Log file 3
```

**File Growth:**
- Files grow indefinitely (no automatic compaction yet)
- Each operation adds one log entry
- File size = sum of all historical operations

#### Thread Safety

- **File Access**: Each file is protected by a `Mutex`
- **Concurrent Writes**: Different keys can write to different files simultaneously
- **Lock Granularity**: Per-file locking minimizes contention
- **Read Safety**: Backing store handles concurrent read access

#### When to Use
- Production deployments requiring data persistence
- Development with data that needs to survive restarts
- Any scenario where durability is more important than write performance

## Comparison

| Feature | InMemoryStore | AppendOnlyLogStore |
|---------|---------------|-------------------|
| **Persistence** | None | Full durability |
| **Write Speed** | Very Fast | Moderate (disk I/O) |
| **Read Speed** | Very Fast | Very Fast |
| **Memory Usage** | Data only | Data + log overhead |
| **Crash Recovery** | Data lost | Full recovery |
| **Startup Time** | Instant | Recovery time |
| **Disk Usage** | None | Growing log files |
| **Concurrency** | Excellent | Good (file sharding) |

## Future Improvements

### InMemoryStore
- **Metrics**: Memory usage tracking
- **Eviction**: LRU or other eviction policies
- **Snapshots**: Point-in-time memory dumps

### AppendOnlyLogStore
- **Log Compaction**: Periodic removal of obsolete entries
- **Checksums**: Corruption detection for log entries
- **Compression**: Reduce disk space usage
- **Parallel Recovery**: Concurrent processing of multiple log files
- **Configurable Sync**: Option to batch writes before fsync
- **Background Compaction**: Automatic log file maintenance
