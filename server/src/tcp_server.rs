//! Implements the command handler trait over tcp

use crate::CommandError;
use crab_core::{Deserialize, Key, Object, slice_to_array};
use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

/// The default port to listen on
const DEFAULT_PORT: u16 = 7227;
/// The default ip address to listen for incomming connectons on
const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
/// The default socket address to listen on
const DEFAULT_SOCKET_ADDRESS: SocketAddr = SocketAddr::new(DEFAULT_IP, DEFAULT_PORT);

/// The data type of the number used to store the command length
type CommandLenType = u64;
/// The number of bytes used to represent the command length
const COMMAND_LEN: usize = std::mem::size_of::<CommandLenType>();

/// The data type of the number used to store the key length
type KeyType = u16;
/// The number of bytes used to represent the key length
const KEY_LEN: usize = std::mem::size_of::<KeyType>();

/// The data type of the number used to store the data type
type CommandOpType = u8;
/// The number of bytes used to represent the command type
const COMMAND_OP_LEN: usize = std::mem::size_of::<CommandOpType>();

/// The struct responsible for handeling commands over tcp
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    // Create a new command handler
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        Self {
            // TODO: Add error handling
            listener: TcpListener::bind(addr).unwrap(),
        }
    }
}

impl Default for TcpServer {
    fn default() -> Self {
        Self::new(DEFAULT_SOCKET_ADDRESS)
    }
}

impl crate::Server for TcpServer {
    type Handler = TcpStream;

    fn listen(&self) -> Self::Handler {
        // TODO: Add error handling
        self.listener.accept().map(|(stream, _)| stream).unwrap()
    }
}

// structure for a command over tcp is
// | 8 bytes command length not including the first 8 bytes | 1 byte command type | rest command specific |

// implementing Connection for a tcp stream
impl crate::Connection for TcpStream {
    fn recieve(&mut self) -> Result<crate::Command, crate::CommandError> {
        // first get the size of the payload
        let mut buffer = [0; COMMAND_LEN];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::CommandError::RecieveFailed)?;
        let command_len = CommandLenType::from_be_bytes(buffer);

        // TODO: Add check to make sure that the command length is long enough to accomodate everything

        // read the rest of the data
        let mut buffer = vec![0; command_len as usize];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::CommandError::RecieveFailed)?;

        crate::Command::try_from(buffer.as_slice())
    }

    fn send(&self) {
        todo!()
    }
}

impl TryFrom<&[u8]> for crate::Command {
    type Error = CommandError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // first determine the tyep of command sent
        let command_type =
            CommandOpType::from_be_bytes(unsafe { slice_to_array(&value[..COMMAND_OP_LEN]) });
        let value = &value[COMMAND_OP_LEN..];

        match command_type {
            // GET
            // | 2 bytes key length (n) | n bytes key |
            0 => Ok(crate::Command::Get(
                Key::deserialize(value)
                    .map(|(key, _)| key)
                    .map_err(|_| CommandError::InvalidKey)?,
            )),
            // SET
            // | 2 bytes key length (n) | n bytes key | 1 byte data type | rest of the data payload |
            1 => {
                let (key, value) = Key::deserialize(value).map_err(|_| CommandError::InvalidKey)?;

                let (object, _) =
                    Object::deserialize(value).map_err(|_| CommandError::InvalidObject)?;

                Ok(crate::Command::Set(key, object))
            }
            _ => Err(CommandError::InvalidType),
        }
    }
}
