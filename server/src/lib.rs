use object::{Key, Object, ObjectError};
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
    /// When an error occurs while building an Object
    Object(object::ObjectError),
    /// The Command requested does not exist
    Invalid(CommandType),
    /// There was an issue with the passed in parameters
    Param,
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
            CommandError::Param => {
                write!(f, "an error occured while evaluating passed in parameters",)
            }
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

/// The parameters to use for link resolution
/// This param has a 1 byte value
#[derive(Debug)]
pub struct LinkResolution {
    /// The maximum number of times link resolution may take place before exiting
    max_resolutions: u8,
}

impl LinkResolution {
    /// Create a new LinkResolution from bytes
    fn new(max_resolutions: u8) -> Self {
        Self { max_resolutions }
    }

    /// Return the max number of resolutions
    pub fn max_resolutions(&self) -> u8 {
        self.max_resolutions
    }

    /// Extract the number of resolutions from bytes to create a LinkResolution and return the remaining bytes
    /// Returns an error if there is not enough bytes
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), CommandError> {
        if bytes.is_empty() {
            Err(CommandError::Param)
        } else {
            // Already verified that it is not empty
            let max_resolutions = unsafe { bytes.get_unchecked(0) };
            Ok((Self::new(*max_resolutions), &bytes[1..]))
        }
    }
}

/// Contains the params that can be sent along with the get command
#[derive(Debug, Default)]
pub struct GetParams {
    /// Whether links should get resolved or not
    // TODO: should this be pub?
    pub link_resolution: Option<LinkResolution>,
}

impl GetParams {
    /// The number associated with link resolution parameter
    const LINK_RESOLUTION_VAL: u8 = 1;

    /// Create new GetParams from bytes
    /// returns a CommandError::Param if there is an erro parsing
    fn from_bytes(data: &[u8]) -> Result<Self, CommandError> {
        let mut params = GetParams::default();

        if data.is_empty() {
            return Ok(params);
        }

        // getting num params
        let num_params = unsafe { *data.get_unchecked(0) }; // we know there is at least 1 element

        let mut data = &data[1..];
        for _ in 0..num_params {
            // check if there is enough data to continue
            if data.is_empty() {
                return Err(CommandError::Param);
            }

            // getting the param type
            let param_type = unsafe { *data.get_unchecked(0) }; // we know there is at least 1 element

            match param_type {
                Self::LINK_RESOLUTION_VAL => {
                    let (link_resolution, rest) = LinkResolution::from_bytes(&data[1..])?;
                    params.link_resolution = Some(link_resolution);
                    data = rest;
                }
                _ => return Err(CommandError::Param),
            }
        }

        Ok(params)
    }
}

/// The kind of commands a client can send
#[derive(Debug)]
pub enum Command {
    /// Get an Object from its Key
    /// Structure is as follows:
    /// | 8 bytes payload size | 1 byte command type | key | 1 byte num params (optional) | 1 byte param type if param num present| n bytes param value| more params |
    /// Params are passed in as a bitflag. Param values are evaluated from lowest bit value to the highest
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
        // extract key
        let (key, rest) = Key::new(data.as_slice())?;
        let params = GetParams::from_bytes(rest)?;
        Ok(Self::Get(key, params))
    }

    /// Creates a new Set command
    fn new_set(data: Vec<u8>) -> Result<Self, CommandError> {
        // first extract Key
        let (key, data) = Key::new(data.as_slice())?;
        Ok(Self::Set(
            key,
            Object::deserialize(data).map(|(obj, _)| obj)?,
        ))
    }

    /// Creates a new Delete command
    fn new_delete(data: Vec<u8>) -> Result<Self, CommandError> {
        // extract key
        let (key, _) = Key::new(data.as_slice())?;
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
        let object = object.serialize();
        // getting legnth of payload
        let payload_size = object.len() as u64;

        // building payload
        let mut payload = payload_size.to_be_bytes().to_vec();
        payload.extend(object);

        self.stream.write_all(&payload)
    }
}
