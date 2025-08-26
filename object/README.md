# Object

Contains the definition of an object in the database as well as the built-in data types for CrabDB.

## Overview

The `object` crate provides CrabDB's type system, including serialization and deserialization of all supported data types. Each object knows its type and can be safely converted between binary and structured representations.

## Supported Data Types

CrabDB supports the following built-in data types:

| Type ID | Type Name | Description | Storage Format |
|---------|-----------|-------------|----------------|
| `0` | **Null** | Represents no value | No data |
| `1` | **Int** | 64-bit signed integer | 8-byte big-endian |
| `2` | **Text** | UTF-8 string | 2-byte length + string data |
| `3` | **List** | Ordered collection of objects | 2-byte count + serialized objects |
| `4` | **Map** | Key-value mapping (string keys) | 2-byte field count + field entries |
| `5` | **Link** | Reference to another object by key | Same format as Key |

## Key Features

- **Type Safety**: Each object carries its type information
- **Recursive Structures**: Lists and Maps can contain any other objects, including nested Lists and Maps
- **Link References**: Link type enables object relationships and references
- **Binary Serialization**: Efficient binary format for network transmission and storage
- **Zero-Copy Deserialization**: Minimal memory allocation during parsing

## Core Components

### Object
The main `Object` struct represents any value in the database:
- Stores type information (`ObjectKind`)
- Contains serialized binary data
- Provides serialization/deserialization methods

### Key
Keys identify objects in the database:
- UTF-8 encoded strings with length prefix
- Used for object storage and Link references
- Consistent binary format across the system

### ObjectKind
Enum representing all supported data types:
- Used for type checking and dispatch
- Convertible to/from binary type IDs
- Enables safe type-specific operations

## Usage Examples

```rust
use object::{Object, ObjectKind, Key};

// Create objects of different types
let null_obj = Object::null();
let int_obj = Object::from(42i64);
let text_obj = Object::from("Hello, World!");

// Serialize for storage/transmission
let bytes = int_obj.serialize();

// Deserialize from binary data
let (restored_obj, remaining) = Object::deserialize(&bytes)?;

// Type checking
match restored_obj.kind() {
    ObjectKind::Int => println!("It's an integer!"),
    ObjectKind::Text => println!("It's text!"),
    _ => println!("It's something else!"),
}
```

## Link Type and References

The Link type enables creating relationships between objects:

```rust
// Create a link to another object
let target_key = Key::from("user:123");
let link_obj = Object::from(Link::from(target_key));

// Links can be resolved by the engine when retrieving objects
// This enables building complex data relationships
```

## Binary Protocol Integration

This crate integrates seamlessly with CrabDB's binary protocol:
- Objects serialize to the exact format expected by the network protocol
- Keys use the same format for object identification and Link references
- Type IDs match the protocol specification exactly
