use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};

/// The server for the database. It listens over TCP
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
    pub fn listen(&self) {
        self.listener.accept();
    }
}

/// Represents a connection to a client
pub struct Connection {
    /// The TcpStream which is connect to the client
    stream: TcpStream,
}
