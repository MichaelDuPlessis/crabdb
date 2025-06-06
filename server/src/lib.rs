//! This module is responsible for handling recieving of commands and the sending of data

use crab_core::{Key, Object};

pub mod tcp_server;

/// The kinds of errors that can occur when recieving a command
#[derive(Debug)]
pub enum CommandError {
    /// The command type sent is invalid
    InvalidType,
    /// Failed to recieve data from client
    RecieveFailed,
    /// The data recieved was not compelete
    Incomplete,
    /// The object provided was invalid such as being malformed or have a lenght of 0
    InvalidObject,
    /// The data type specified is invalid
    InvalidDataType,
    /// The key is invalid such as not being a valid utf8 string
    InvalidKey,
}

/// A command sent by a client
#[derive(Debug)]
pub enum Command {
    /// Get data from a specific key
    // | 2 bytes key length (n) | n bytes key |
    Get(Key),
    /// Sets data on a specific key
    // | 2 bytes key length (n) | n bytes key | 1 byte data type | rest of the bytes data |
    Set(Key, Object),
    /// The connection is closed
    Terminated,
}

// Structure of command
/// All methods that a command handler must implement to be usable
pub trait Server {
    type Handler: Connection;

    /// The CommandHandler just needs to be able to listen for commands
    /// it is then responsible for processing the commands and returning a response
    fn listen(&self) -> Self::Handler;
}

/// Represents some connection to the server
pub trait Connection {
    /// Retrieve command from a connection from a client. This blocks until a command is received or the connection is closed
    fn recieve(&mut self) -> Result<Command, CommandError>;

    /// Send data to connection
    fn send(&self);
}
