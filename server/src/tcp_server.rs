//! Implements the Server trait over tcp

use crate::RecieveError;
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

/// The data type of the number used to store the request length
type RequestLenType = u64;
/// The number of bytes used to represent the request length
const REQUEST_LEN: usize = std::mem::size_of::<RequestLenType>();

/// The data type of the number used to store the data type
type RequestOpType = u8;
/// The number of bytes used to represent the requests type
const REQUEST_OP_LEN: usize = std::mem::size_of::<RequestOpType>();

/// The struct responsible for handeling connections over tcp
pub struct TcpServer {
    listener: TcpListener,
}

impl TcpServer {
    // Create a new tcp server
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

// structure for a request over tcp is
// | 8 bytes request length not including the first 8 bytes | 1 byte request type | rest request specific |

// implementing Connection for a tcp stream
impl crate::Connection for TcpStream {
    fn receive(&mut self) -> Result<crate::Request, crate::RecieveError> {
        // first get the size of the payload
        let mut buffer = [0; REQUEST_LEN];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::RecieveError::RecieveFailed)?;
        let request_len = RequestLenType::from_be_bytes(buffer);

        // TODO: Add check to make sure that the request length is long enough to accomodate everything

        // read the rest of the data
        let mut buffer = vec![0; request_len as usize];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::RecieveError::RecieveFailed)?;

        crate::Request::try_from(buffer.as_slice())
    }

    fn send(&self, response: crate::Response) -> Result<(), crate::ResponseError> {
        todo!()
    }
}

impl TryFrom<&[u8]> for crate::Request {
    type Error = RecieveError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // first determine the tyep of request sent
        let request_type =
            RequestOpType::from_be_bytes(unsafe { slice_to_array(&value[..REQUEST_OP_LEN]) });
        let value = &value[REQUEST_OP_LEN..];

        match request_type {
            // GET
            // | 2 bytes key length (n) | n bytes key |
            0 => Ok(crate::Request::Get(
                Key::deserialize(value)
                    .map(|(key, _)| key)
                    .map_err(|_| RecieveError::InvalidKey)?,
            )),
            // SET
            // | 2 bytes key length (n) | n bytes key | 1 byte data type | rest of the data payload |
            1 => {
                let (key, value) = Key::deserialize(value).map_err(|_| RecieveError::InvalidKey)?;

                let (object, _) =
                    Object::deserialize(value).map_err(|_| RecieveError::InvalidObject)?;

                Ok(crate::Request::Set(key, object))
            }
            _ => Err(RecieveError::InvalidType),
        }
    }
}
