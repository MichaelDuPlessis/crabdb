//! Implements the command handler trait over tcp

use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

/// The struct responsible for handeling commands over tcp
pub struct TcpCommandHandler {
    listener: TcpListener,
}

impl crate::CommandHandler for TcpCommandHandler {
    fn listen(&self) -> impl crate::Connection {
        // TODO: Add error handling
        self.listener.accept().map(|(stream, _)| stream).unwrap()
    }
}

// implementing Connection for a tcp stream
impl crate::Connection for TcpStream {
    fn recieve(&mut self) -> Result<crate::Command, crate::RecieveError> {
        // buffer to read into
        // TODO: Use with_capacity instead
        let buffer = &mut Vec::new();

        let _ = self
            .read_to_end(buffer)
            .map_err(|_| crate::RecieveError::RecieveFailed)?;

        todo!()
    }

    fn send(&self) {
        todo!()
    }
}
