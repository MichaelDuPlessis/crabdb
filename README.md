# CrabDB

CrabDB is a database designed to be easy to use and quick to iterate on. It is not meant for high performance workloads
and is instead meant to be used for prototyping or workloads that are not going to be "stress tested".

## Goal

The goal of this project is to create an easy to use NoSQL database that feels like querying a json object. It is meant to be used
in small projects or for prototyping.

Another goal of this project is to use as little external dependencies as possible and have everything be coded from scratch as this
is primarily (at least for now) just a fun little side project to learn various things.

## TCP Protocol

Interaction with the database is done over a TCP connection. All integer values in the protocol are sent in **big-endian** format.

### General Request Structure

Every request sent to the server follows this basic structure:

| Part                  | Size (bytes) | Description                                         |
| --------------------- | ------------ | --------------------------------------------------- |
| Request Length        | 8            | The total length of the following data (in bytes).  |
| Request Type          | 1            | The command to execute (`0` for GET, `1` for SET).  |
| Request Specific Data | variable     | The payload, which depends on the `Request Type`.   |

### Commands

#### `SET` Command (Request Type: `1`)

The `SET` command is used to store a key-value pair.

**Payload Structure:**

| Part          | Size (bytes) | Description                                                |
| ------------- | ------------ | ---------------------------------------------------------- |
| Key Length    | 2            | The length of the key string (`n`).                        |
| Key           | `n`          | The key, encoded in UTF-8.                                 |
| Data Type     | 1            | The type of data being stored (`0`=Null, `1`=Int, `2`=Text). |
| Data Payload  | variable     | The actual data, structured according to its `Data Type`.  |


#### `GET` Command (Request Type: `0`)

The `GET` command is used to retrieve a value by its key.

**Payload Structure:**

| Part       | Size (bytes) | Description                   |
| ---------- | ------------ | ----------------------------- |
| Key Length | 2            | The length of the key (`n`).  |
| Key        | `n`          | The key, encoded in UTF-8.    |

### Data Types

The database supports the following data types, each with its own binary structure.

| Type ID | Type Name |
| ------- | --------- |
| `0`     | Null      |
| `1`     | Int       |
| `2`     | Text      |

#### `Null` (Type: `0`)

A null value has no data payload.

#### `Int` (Type: `1`)

An integer is stored as a signed 8-byte integer.

| Part   | Size (bytes) | Description             |
| ------ | ------------ | ----------------------- |
| Number | 8            | A signed 64-bit integer.|


#### `Text` (Type: `2`)

A text value is prefixed with its length.

| Part        | Size (bytes) | Description                   |
| ----------- | ------------ | ----------------------------- |
| Text Length | 2            | The length of the text (`m`). |
| Text Data   | `m`          | The text, encoded in UTF-8.   |
