# CrabDB

A lightweight, educational NoSQL database written in Rust. CrabDB is designed for learning database internals and rapid prototyping, not high-performance production workloads.

## Architecture

CrabDB is built as a modular Rust workspace with the following components:

- **`engine`** - Main server binary that coordinates all components
- **`server`** - TCP server handling client connections and protocol parsing
- **`object`** - Type system and serialization for database values
- **`storage`** - Storage backends (currently in-memory only)
- **`concurrent-map`** - Thread-safe hash map implementation
- **`threadpool`** - Custom thread pool for handling concurrent connections
- **`logging`** - Custom logging system
- **`client`** - Python CLI client for testing and interaction

## Features

- **TCP-based protocol** with binary serialization
- **Concurrent connections** using a custom thread pool
- **Type-safe data storage** with Null, Int (i64), and Text types
- **Thread-safe in-memory storage** using a custom concurrent hash map
- **Zero external dependencies** - everything built from scratch for learning

## Getting Started

### Running the Server

```bash
# Build and run the database engine
cargo run --bin engine
```

The server listens on port `7227` by default.

### Using the Python Client

```bash
# Interactive mode
python3 client/cli.py

# Direct commands
python3 client/cli.py set mykey int 42
python3 client/cli.py get mykey
```

## Protocol Specification

CrabDB uses a custom binary protocol over TCP. All integers are sent in **big-endian** format.

### Request Structure

| Field           | Size (bytes) | Description                                    |
|-----------------|--------------|------------------------------------------------|
| Request Length  | 8            | Total length of following data                 |
| Request Type    | 1            | Command type (`0`=GET, `1`=SET, `255`=CLOSE)  |
| Request Data    | variable     | Command-specific payload                       |

### Commands

#### GET Command (Type: `0`)
Retrieve a value by key.

**Request Payload:**
| Field      | Size (bytes) | Description           |
|------------|-------------|-----------------------|
| Key Length | 2           | Length of key string  |
| Key        | variable    | UTF-8 encoded key     |

#### SET Command (Type: `1`)
Store a key-value pair.

**Request Payload:**
| Field      | Size (bytes) | Description                    |
|------------|-------------|--------------------------------|
| Key Length | 2           | Length of key string           |
| Key        | variable    | UTF-8 encoded key              |
| Data Type  | 1           | Value type (`0`/`1`/`2`)       |
| Data       | variable    | Type-specific value data       |

#### CLOSE Command (Type: `255`)
Close the connection. No payload required.

### Data Types

| Type ID | Type Name | Storage Format                           |
|---------|-----------|------------------------------------------|
| `0`     | Null      | No data                                  |
| `1`     | Int       | 8-byte signed integer (big-endian)      |
| `2`     | Text      | 2-byte length + UTF-8 string data       |

### Response Structure

**Success Response:**
| Field           | Size (bytes) | Description                    |
|-----------------|--------------|--------------------------------|
| Response Length | 8            | Total length of following data |
| Data Type       | 1            | Type of returned value         |
| Data            | variable     | Type-specific value data       |

**Error Response:**
| Field           | Size (bytes) | Description                    |
|-----------------|--------------|--------------------------------|
| Response Length | 8            | Always `1` for errors          |
| Error Code      | 1            | Always `255` (generic error)   |

## Implementation Details

### Type System
The `object` crate provides a strongly-typed system where each value knows its type. Values are serialized with a type prefix, enabling type-safe deserialization.

### Concurrency
- Custom thread pool handles multiple client connections
- Thread-safe storage using a custom concurrent hash map
- Each connection runs in its own thread with shared storage access

### Storage
Currently implements in-memory storage only. The `Store` trait allows for pluggable storage backends (disk persistence, etc.).

### Error Handling
- Comprehensive error types for network, parsing, and object construction failures
- Graceful connection handling with proper cleanup

## Development Goals

This project prioritizes:
1. **Educational value** - Learn database internals by building from scratch
2. **Minimal dependencies** - Custom implementations of core components
3. **Clean architecture** - Modular design with clear separation of concerns
4. **Type safety** - Leverage Rust's type system for correctness

## Limitations

- **In-memory only** - No persistence across restarts
- **Simple protocol** - No authentication, compression, or advanced features
- **Basic error handling** - Generic error responses
- **No indexing** - Linear search for non-key operations
- **Limited types** - Only Null, Int, and Text supported

## Future Enhancements

- Disk-based persistence
- Additional data types (Boolean, Float, List, Object)
- Query language for filtering and sorting
- Secondary indexes
- Compression and authentication
- Replication and clustering

---

*CrabDB is a learning project focused on understanding database fundamentals. It's not intended for production use.*
