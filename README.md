# CrabDB

A lightweight, NoSQL database written in Rust. CrabDB is designed for rapid prototyping or lightweight workloads, not high-performance production workloads.

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
- **Type-safe data storage** with Null, Int (i64), Text, List, and Map types
- **Thread-safe in-memory storage** using a custom concurrent hash map
- **Zero external dependencies** - everything built from scratch for learning

## Getting Started

### Running the Server

#### With Cargo (Development)

```bash
# Build and run the database engine
cargo run --bin engine
```

#### With Docker (Production)

```bash
# Build the Docker image
docker build -t crabdb .

# Run the container with data persistence
docker run -p 7227:7227 -v ./data:/app/data crabdb

# Run in background (detached mode)
docker run -d -p 7227:7227 -v ./data:/app/data --name crabdb-server crabdb

# Stop the container
docker stop crabdb-server

# Remove the container
docker rm crabdb-server
```

The server listens on port `7227` by default.

### Using the Python Client

```bash
# Interactive mode
python3 client/cli.py

# Direct commands
python3 client/cli.py set mykey int 42
python3 client/cli.py set mytext text "hello world"
python3 client/cli.py set mylist list '[1, "hello", null]'
python3 client/cli.py set mymap map '{"name": "John", "age": 30, "active": null}'
python3 client/cli.py set nested map '{"user": {"name": "Alice", "scores": [95, 87, 92]}, "meta": {"created": "2024-01-01"}}'
python3 client/cli.py set complex list '[{"id": 1, "tags": ["red", "blue"]}, {"id": 2, "tags": ["green"]}]'
python3 client/cli.py get mykey
python3 client/cli.py get mylist
python3 client/cli.py get mymap
python3 client/cli.py get nested
python3 client/cli.py close
python3 client/cli.py del mykey
```

**Note**: List and Map values use JSON format:
- **Lists**: `'[1, "hello", null]'` - JSON array with mixed types
- **Maps**: `'{"name": "John", "age": 30, "active": null}'` - JSON object with string keys
- **Nested structures**: Full nesting support - Lists can contain Maps, Maps can contain Lists, etc.

**Interactive Mode**: 
- Use `python3 client/cli.py --interactive` for persistent connections
- Press **Ctrl+C** to gracefully send a close request and exit
- Use `close` command to send shutdown request to server
- Use `exit` or `quit` to close the client (also sends close request if connected)

## Protocol Specification

CrabDB uses a custom binary protocol over TCP. All integers are sent in **big-endian** format.

### Request Structure

| Field           | Size (bytes) | Description                                    |
|-----------------|--------------|------------------------------------------------|
| Request Length  | 8            | Total length of following data                 |
| Request Type    | 1            | Command type                                   |
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
| Data Type  | 1           | Value type (`0`/`1`/`2`/`3`/`4`) |
| Data       | variable    | Type-specific value data       |

#### DELETE Command (Type: `2`)
Delete a value by key.

**Request Payload:**
| Field      | Size (bytes) | Description           |
|------------|-------------|-----------------------|
| Key Length | 2           | Length of key string  |
| Key        | variable    | UTF-8 encoded key     |

#### CLOSE Command (Type: `255`)
Close the connection. No payload required.

### Data Types

| Type ID | Type Name | Storage Format                           |
|---------|-----------|------------------------------------------|
| `0`     | Null      | No data                                  |
| `1`     | Int       | 8-byte signed integer (big-endian)      |
| `2`     | Text      | 2-byte length + UTF-8 string data       |
| `3`     | List      | 2-byte count + serialized objects       |
| `4`     | Map       | 2-byte field count + field entries      |

#### List Format (Type ID: `3`)
| Field        | Size (bytes) | Description                    |
|--------------|--------------|--------------------------------|
| Object Count | 2            | Number of objects in the list  |
| Objects      | variable     | Serialized objects in sequence |

#### Map Format (Type ID: `4`)
| Field       | Size (bytes) | Description                     |
|-------------|-------------|---------------------------------|
| Field Count | 2           | Number of key-value pairs       |
| Fields      | variable    | Field entries in sequence       |

**Field Entry Format:**
| Field           | Size (bytes) | Description                    |
|-----------------|--------------|--------------------------------|
| Field Name Len  | 2            | Length of field name           |
| Field Name      | variable     | UTF-8 encoded field name       |
| Object          | variable     | Serialized object value        |

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
Currently implements and in-memory storage and an append only log store. The `Store` trait allows for pluggable storage backends (disk persistence, etc.).

### Error Handling
- Comprehensive error types for network, parsing, and object construction failures
- Graceful connection handling with proper cleanup
