//! This module is responsible for handling recieving of commands and the sending of data

use std::io::BufRead;

mod tcp_command_handler;

/// The kinds of errors that can occur when recieving a command
#[derive(Debug)]
pub enum CommandError {
    /// The command type requested is invalid
    InvalidCommandType,
    /// Failed to recieve data from client
    RecieveFailed,
    /// No data was recieved
    ZeroLen,
    /// The object provided was invalid such as being malformed or have a lenght of 0
    InvalidObject,
    /// The data type specified is invalid
    InvalidDataType,
    /// The key is invalid such as not being a valid utf8 string
    InvalidKey,
}

/// A key used to identify an object in the DB
#[derive(Debug)]
pub struct Key(String);

impl TryFrom<&[u8]> for Key {
    type Error = CommandError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        str::from_utf8(value)
            .map(|s| Self(s.to_string()))
            .map_err(|_| Self::Error::InvalidKey)
    }
}

/// An object stored in the database
#[derive(Debug)]
pub enum Object {
    Int(i64),
}

impl TryFrom<&[u8]> for Object {
    type Error = CommandError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // if the slice is empty than no Object can be derived from it
        if value.is_empty() {
            return Err(CommandError::InvalidObject);
        }

        // first determine what data type is being used
        let d_type = value[0];

        match d_type {
            0 => Ok(Self::Int(i64::from_be_bytes(
                value[1..]
                    .try_into()
                    .map_err(|_| Self::Error::InvalidObject)?,
            ))),
            _ => Err(Self::Error::InvalidDataType),
        }
    }
}

/// A command sent by a client
#[derive(Debug)]
pub enum Command {
    /// Get data from a specific key
    Get(Key),
    /// Sets data on a specific key
    Set(Key, Object),
}

impl TryFrom<&[u8]> for Command {
    type Error = CommandError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(Self::Error::ZeroLen);
        }

        // splitting the byte input by " "
        let inputs = value.split(|v| *v == b' ');

        todo!()
    }
}

/// All methods that a command handler must implement to be usable
pub trait CommandHandler {
    /// The CommandHandler just needs to be able to listen for commands
    /// it is then responsible for processing the commands and returning a response
    fn listen(&self) -> impl Connection;
}

/// Represents some connection to the server
pub trait Connection {
    /// Retrieve command from a connection from a client
    fn recieve(&mut self) -> Result<Command, CommandError>;

    /// Send data to connection
    fn send(&self);
}
