//! This module is responsible for handling recieving of commands and the sending of data

pub mod tcp_command_handler;

/// The kinds of errors that can occur when recieving a command
#[derive(Debug)]
pub enum CommandError {
    /// The command type requested is invalid
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
        let d_type = DataTypeType::from_be_bytes(value[..DATA_TYPE_LEN].try_into().unwrap());

        match d_type {
            0 => Ok(Self::Int(i64::from_be_bytes(
                value[DATA_TYPE_LEN..]
                    .try_into()
                    .map_err(|_| Self::Error::InvalidObject)?,
            ))),
            _ => Err(Self::Error::InvalidDataType),
        }
    }
}

/// The data type of the number used to store the command length
type CommandType = u64;
/// The number of bytes used to represent the command length
const COMMAND_LEN: usize = std::mem::size_of::<CommandType>();

/// The data type of the number used to store the key length
type KeyType = u16;
/// The number of bytes used to represent the key length
const KEY_LEN: usize = std::mem::size_of::<KeyType>();

/// The data type of the number used to store the data type
type CommandOpType = u8;
/// The number of bytes used to represent the command type
const COMMAND_OP_LEN: usize = std::mem::size_of::<CommandOpType>();

/// The data type of the number used to store the data type
type DataTypeType = u8;
/// The number of bytes used to represent the data type
const DATA_TYPE_LEN: usize = std::mem::size_of::<DataTypeType>();

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

impl Command {
    /// Create a Get command from a &[u8]
    fn get_from_slices<'a>(slice: &'a [u8]) -> Result<Self, <Self as TryFrom<&'a [u8]>>::Error> {
        // first get the key length
        let key_length = KeyType::from_be_bytes(slice[..KEY_LEN].try_into().unwrap());
        // getting the key
        let key = Key::try_from(&slice[..key_length as usize])?;

        Ok(Self::Get(key))
    }

    /// Create a Set command from a &[u8]
    fn set_from_slices<'a>(slice: &'a [u8]) -> Result<Self, <Self as TryFrom<&'a [u8]>>::Error> {
        // first get the key length
        let key_length = KeyType::from_be_bytes(slice[..KEY_LEN].try_into().unwrap()) as usize;
        // getting the key
        let key = Key::try_from(&slice[..key_length])?;

        // getting the object
        let object = Object::try_from(&slice[KEY_LEN + key_length..])?;

        Ok(Self::Set(key, object))
    }
}

// Structure of command
// | 8 bytes length of data to be received including these bytes | 1 byte command type |
impl TryFrom<&[u8]> for Command {
    type Error = CommandError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // getting command type byte
        // it is assumed it is valid to index
        let command_type =
            CommandOpType::from_be_bytes(value[..COMMAND_OP_LEN].try_into().unwrap());

        match command_type {
            0 => Self::get_from_slices(&value[COMMAND_OP_LEN..]),
            1 => Self::set_from_slices(&value[COMMAND_OP_LEN..]),
            2 => Ok(Self::Terminated),
            _ => Err(Self::Error::InvalidType),
        }
    }
}

/// All methods that a command handler must implement to be usable
pub trait CommandHandler {
    type Handler: Connection;

    /// The CommandHandler just needs to be able to listen for commands
    /// it is then responsible for processing the commands and returning a response
    fn listen(&self) -> Self::Handler;
}

/// Represents some connection to the server
pub trait Connection {
    /// Retrieve command from a connection from a client
    fn recieve(&mut self) -> Result<Command, CommandError>;

    /// Send data to connection
    fn send(&self);
}
