use object::{Key, ObjectError};
use std::{
    fmt,
    io::{self, Read},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

/// The server for the database. It listens over TCP
#[derive(Debug)]
pub struct Server {
    /// The open socket
    listener: TcpListener,
}

impl Server {
    /// Create a new server
    pub fn new(port: u16) -> Self {
        // if it is not possilbe to bind to a port just fail
        let listener =
            TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port))
                .expect("Failed to bind to port: {port}");

        Self { listener }
    }

    /// Wait for a connnection to be recieved
    pub fn listen(&self) -> Result<Connection, io::Error> {
        let (stream, _) = self.listener.accept()?;

        Ok(Connection::new(stream))
    }
}

/// The size of the payload in bytes
type PayloadSize = u64;
/// The Number of bytes PayloadSize requires
const PAYLOAD_SIZE_NUM_BYTES: usize = std::mem::size_of::<PayloadSize>();

/// The size of the payload in bytes
type CommandType = u8;
/// The Number of bytes PayloadSize requires
const COMMAND_TYPE_NUM_BYTES: usize = std::mem::size_of::<CommandType>();

/// The types of errors that can occur when building a Command
#[derive(Debug)]
pub enum CommandError {
    /// When an error occurs with the underlying network
    Network(io::Error),
    /// When an error occurs while building an Object
    Object(object::ObjectError),
    /// The Command requested does not exist
    Invalid(CommandType),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Network(error) => write!(f, "a network error occured {}", error),
            CommandError::Object(object_error) => {
                write!(f, "unable to create object {}", object_error)
            }
            CommandError::Invalid(command_type) => write!(
                f,
                "the command type sent was invalid, received: {}",
                command_type
            ),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<io::Error> for CommandError {
    fn from(value: io::Error) -> Self {
        Self::Network(value)
    }
}

impl From<object::ObjectError> for CommandError {
    fn from(value: object::ObjectError) -> Self {
        Self::Object(value)
    }
}

/// The kind of commands a client can send
#[derive(Debug)]
pub enum Command {
    /// Get an Object from its Key
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key |
    Get(Key),
    /// Create an Object in the DB
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key | data object |
    Set(Key, Vec<u8>),
}

impl Command {
    /// The value for the Get Command
    const GET: u8 = 0;
    /// The value for the Set Command
    const SET: u8 = 1;

    /// Creats a new command based on the CommandType and the data
    pub fn new(command_type: CommandType, data: Vec<u8>) -> Result<Self, CommandError> {
        match command_type {
            Self::GET => Self::new_get(data),
            Self::SET => Self::new_set(data),
            _ => return Err(CommandError::Invalid(command_type)),
        }
        .map_err(|err| CommandError::Object(err))
    }

    /// Creates a new Get command
    fn new_get(data: Vec<u8>) -> Result<Self, ObjectError> {
        // extract key
        let (key, _) = Key::new(data.as_slice())?;
        Ok(Self::Get(key))
    }

    /// Creates a new Set command
    fn new_set(data: Vec<u8>) -> Result<Self, ObjectError> {
        // first extract Key
        let (key, data) = Key::new(data.as_slice())?;
        // extract Object next

        todo!()
    }
}

/// Represents a connection to a client
pub struct Connection {
    /// The TcpStream which is connect to the client
    stream: TcpStream,
}

impl Connection {
    /// Creates a new connection from TcpStream
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Recieves a command from the client
    pub fn recieve(&mut self) -> Result<Command, CommandError> {
        // first receive the number of bytes being sent
        let mut buffer = [0; PAYLOAD_SIZE_NUM_BYTES];
        self.stream.read_exact(&mut buffer)?;
        let payload_size = PayloadSize::from_be_bytes(buffer);

        // recieve the command type
        // TODO: I know this code is duplicatd I am still trying to decide if I should use a macro, a common trait or just leave it
        let mut buffer = [0; COMMAND_TYPE_NUM_BYTES];
        self.stream.read_exact(&mut buffer)?;
        let command_type = CommandType::from_be_bytes(buffer);

        // reading the rest of the data
        let mut data = vec![0; payload_size as usize - COMMAND_TYPE_NUM_BYTES];
        self.stream.read_exact(&mut data)?;

        Command::new(command_type, data)
    }
}
