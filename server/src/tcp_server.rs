//! Implements the command handler trait over tcp

use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

/// The maximum number of bytes that can be read
const BUFFER_SIZE: usize = 1024;

/// The default port to listen on
const DEFAULT_PORT: u16 = 7227;
/// The default ip address to listen for incomming connectons on
const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
/// The default socket address to listen on
const DEFAULT_SOCKET_ADDRESS: SocketAddr = SocketAddr::new(DEFAULT_IP, DEFAULT_PORT);

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

// implementing Connection for a tcp stream
impl crate::Connection for TcpStream {
    fn recieve(&mut self) -> Result<crate::Command, crate::CommandError> {
        // buffer to read into
        let mut buffer = [0; BUFFER_SIZE];

        self.read_exact(&mut buffer[..crate::COMMAND_LEN])
            .map_err(|_| crate::CommandError::RecieveFailed)?;

        let command_len =
            crate::CommandType::from_be_bytes(buffer[..crate::COMMAND_LEN].try_into().unwrap())
                as usize;

        // read until all the data from the command has been recieved
        self.read_exact(&mut buffer[..command_len])
            .map_err(|_| crate::CommandError::RecieveFailed)?;

        crate::Command::try_from(&buffer[..command_len])
    }

    fn send(&self) {
        todo!()
    }
}
