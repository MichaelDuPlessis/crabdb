use object::{Key, Object};
use autocodec::Codec;
use std::{
    fmt,
    io::{self, Read, Write},
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
    /// When an error occurs while decoding an Object or Key
    Codec(autocodec::CodecError),
    /// The Command requested does not exist
    Invalid(CommandType),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Network(error) => write!(f, "a network error occured {}", error),
            CommandError::Codec(error) => {
                write!(f, "unable to decode: {}", error)
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

impl From<autocodec::CodecError> for CommandError {
    fn from(value: autocodec::CodecError) -> Self {
        Self::Codec(value)
    }
}

/// The parameters to use for link resolution
/// This param has a 1 byte value
#[derive(Debug)]
pub struct LinkResolution {
    /// The maximum number of times link resolution may take place before exiting
    max_resolution_depth: u8,
}

impl LinkResolution {
    /// Return the max number of resolutions
    pub fn max_resolution_depth(&self) -> u8 {
        self.max_resolution_depth
    }
}

/// A GET command parameter
#[derive(Debug, Codec)]
#[repr(u8)]
pub enum GetParam {
    LinkResolution(u8) = 1,
}

/// Contains the params sent with a GET command
#[derive(Debug, Default, Codec)]
pub struct GetParams {
    #[codec(trailing, len = "u8")]
    pub params: Vec<GetParam>,
}

impl GetParams {
    pub fn link_resolution(&self) -> Option<LinkResolution> {
        self.params.iter().find_map(|p| match p {
            GetParam::LinkResolution(depth) => Some(LinkResolution { max_resolution_depth: *depth }),
        })
    }
}

/// The kind of commands a client can send
#[derive(Debug)]
pub enum Command {
    /// Get an Object from its Key
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key | 1 byte num params (optional) | 1 byte param type if param num present| n bytes param value| more params |
    /// Params are passed in as a bitflag. Param values are evaluated from lowest bit value to the highest
    // TODO: the number of parameters passed in isn't really required but it does allow for further extensibility later on. At some point I should
    // decide if its needed
    Get(Key, GetParams),
    /// Create an Object in the DB
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key | data object |
    Set(Key, Object),
    /// Delete an Object from its Key
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key |
    Delete(Key),
    /// Close the connection to the client
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type |
    Close,
}

impl Command {
    /// The value for the Get Command
    const GET: CommandType = 0;
    /// The value for the Set Command
    const SET: CommandType = 1;
    /// The value for the Delete Command
    const DELETE: CommandType = 2;
    /// Value for the Close Command
    const CLOSE: CommandType = 255;

    /// Creats a new command based on the CommandType and the data
    pub fn new(command_type: CommandType, data: Vec<u8>) -> Result<Self, CommandError> {
        match command_type {
            Self::GET => Self::new_get(data),
            Self::SET => Self::new_set(data),
            Self::DELETE => Self::new_delete(data),
            Self::CLOSE => Ok(Self::Close),
            _ => Err(CommandError::Invalid(command_type)),
        }
    }

    /// Creates a new Get command
    fn new_get(data: Vec<u8>) -> Result<Self, CommandError> {
        let (key, rest) = Key::decode(data.as_slice())?;
        let (params, _) = GetParams::decode(rest)?;
        Ok(Self::Get(key, params))
    }

    /// Creates a new Set command
    fn new_set(data: Vec<u8>) -> Result<Self, CommandError> {
        let (key, rest) = Key::decode(data.as_slice())?;
        let (object, _) = Object::decode(rest)?;
        Ok(Self::Set(key, object))
    }

    /// Creates a new Delete command
    fn new_delete(data: Vec<u8>) -> Result<Self, CommandError> {
        let (key, _) = Key::decode(data.as_slice())?;
        Ok(Self::Delete(key))
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
        let mut payload_size_buffer = [0; PAYLOAD_SIZE_NUM_BYTES];
        self.stream.read_exact(&mut payload_size_buffer)?;
        let payload_size = PayloadSize::from_be_bytes(payload_size_buffer);

        // read the entire payload in one syscall
        let mut payload_buffer = vec![0; payload_size as usize];
        self.stream.read_exact(&mut payload_buffer)?;

        // extract command type from the payload
        if payload_buffer.is_empty() {
            return Err(CommandError::Invalid(0));
        }
        let command_type = payload_buffer[0];

        // extract the data portion (everything after command type)
        let data = payload_buffer[COMMAND_TYPE_NUM_BYTES..].to_vec();

        Command::new(command_type, data)
    }

    /// Sends data back to the client
    pub fn send(&mut self, object: Object) -> Result<(), io::Error> {
        let payload_size = object.encoded_size() as u64;

        let mut payload = Vec::with_capacity(8 + payload_size as usize);
        payload.extend_from_slice(&payload_size.to_be_bytes());
        object.encode(&mut payload);

        self.stream.write_all(&payload)
    }
}
