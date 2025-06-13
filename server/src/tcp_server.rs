//! Implements the Server trait over tcp

use crate::RecieveError;
use crab_core::{Deserialize, Key, Object, Serialize, slice_to_array};
use logging::{debug, info, trace};
use std::{
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

/// The default port to listen on
const DEFAULT_PORT: u16 = 7227;
/// The default ip address to listen for incomming connectons on
const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
/// The default socket address to listen on
const DEFAULT_SOCKET_ADDRESS: SocketAddr = SocketAddr::new(DEFAULT_IP, DEFAULT_PORT);

/// The data type of the number used to store the request length
type DataLenType = u64;
/// The number of bytes used to represent the request length
const DATA_LEN_NUM_BYTES: usize = std::mem::size_of::<DataLenType>();

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

// implementing Connection for a tcp stream
impl crate::Connection for TcpStream {
    fn receive(&mut self) -> Result<crate::Request, crate::RecieveError> {
        // first get the size of the payload
        let mut buffer = [0; DATA_LEN_NUM_BYTES];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::RecieveError::RecieveFailed)?;
        let request_len = DataLenType::from_be_bytes(buffer);

        // TODO: Add check to make sure that the request length is long enough to accomodate everything

        // read the rest of the data
        let mut buffer = vec![0; request_len as usize];
        self.read_exact(&mut buffer)
            .map_err(|_| crate::RecieveError::RecieveFailed)?;

        crate::Request::try_from(buffer.as_slice())
    }

    fn send(&mut self, response: crate::Response) -> Result<(), crate::ResponseError> {
        let response = response
            .serialize()
            .map_err(|_| crate::ResponseError::ResponseFailed)?;

        self.write_all(&response)
            .map_err(|_| crate::ResponseError::ResponseFailed)
    }
}

// structure for a request over tcp is
// | 8 bytes request length not including the first 8 bytes | 1 byte request type | rest request specific |
impl TryFrom<&[u8]> for crate::Request {
    type Error = RecieveError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        // first determine the type of request sent
        debug!("Waiting for request");
        let request_type =
            RequestOpType::from_be_bytes(unsafe { slice_to_array(&value[..REQUEST_OP_LEN]) });
        let value = &value[REQUEST_OP_LEN..];

        match request_type {
            // GET
            // | 2 bytes key length (n) | n bytes key |
            0 => {
                trace!("Get request recieved");

                let key = match Key::deserialize(value) {
                    Ok((key, _)) => {
                        info!("Recieved key: {key:?}");
                        key
                    }
                    Err(_) => return Err(RecieveError::InvalidKey),
                };

                Ok(crate::Request::Get(key))
            }
            // SET
            // | 2 bytes key length (n) | n bytes key | 1 byte data type | rest of the data payload |
            1 => {
                trace!("Set request recieved");
                let key = match Key::deserialize(value) {
                    Ok((key, _)) => {
                        info!("Recieved key: {key:?}");
                        key
                    }
                    Err(_) => return Err(RecieveError::InvalidKey),
                };

                let (object, _) =
                    Object::deserialize(value).map_err(|_| RecieveError::InvalidObject)?;
                info!("Recieved object: {object:?}");

                Ok(crate::Request::Set(key, object))
            }
            _ => Err(RecieveError::InvalidType),
        }
    }
}

// structure for a response over tcp is
// | 8 bytes request length not including the first 8 bytes | 1 byte data type | rest is the data returend |
impl Serialize<Vec<u8>> for crate::Response {
    fn serialize(self) -> Result<Vec<u8>, crab_core::SerializeError> {
        match self {
            crate::Response::Payload(object) => Ok({
                let object = object.serialize()?;
                let mut res = object.len().to_be_bytes().to_vec();
                res.extend(object);
                res
            }),
            // Returning error
            crate::Response::Error => Ok({
                let data_len = (1 as DataLenType).to_be_bytes();
                let mut res = [0; 9];

                res[..DATA_LEN_NUM_BYTES].copy_from_slice(&data_len);
                res[DATA_LEN_NUM_BYTES] = 255;
                res.to_vec()
            }),
        }
    }
}
