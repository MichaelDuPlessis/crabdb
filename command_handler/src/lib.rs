//! This module is responsible for handling recieving of commands and the sending of data

mod tcp_command_handler;

/// The kinds of errors that can occur when recieving a command
pub enum RecieveError {
    /// The command type requested is invalid
    InvalidCommandType,
    /// Failed to recieve data from client
    RecieveFailed,
    /// No data was recieved
    ZeroLen,
}

/// The type of command that is being sent.
/// it is always 1 byte
enum CommandType {
    /// Get data from a specific key
    Get,
    /// Sets data on a specific key
    Set,
}

impl TryFrom<u8> for CommandType {
    type Error = RecieveError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Get),
            1 => Ok(Self::Set),
            _ => Err(<Self as TryFrom<u8>>::Error::InvalidCommandType),
        }
    }
}

/// A command sent by a client
pub struct Command {
    command_type: CommandType,
}

impl TryFrom<&[u8]> for Command {
    type Error = RecieveError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let bytes = value.as_ref();

        if bytes.is_empty() {
            return Err(<Self as TryFrom<&[u8]>>::Error::ZeroLen);
        }

        let command_type = bytes[0].try_into()?;

        Ok(Self { command_type })
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
    fn recieve(&mut self) -> Result<Command, RecieveError>;

    /// Send data to connection
    fn send(&self);
}
