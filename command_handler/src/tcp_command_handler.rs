//! Implements the command handler trait over tcp

use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

/// The maximum number of bytes that can be read
const BUFFER_SIZE: usize = 1024;

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
    fn recieve(&mut self) -> Result<crate::Command, crate::CommandError> {
        // buffer to read into
        let mut buffer = [0; BUFFER_SIZE];

        let _ = self
            .read_exact(&mut buffer[..crate::COMMAND_LEN])
            .map_err(|_| crate::CommandError::RecieveFailed)?;

        let command_len =
            crate::CommandType::from_be_bytes(buffer[..crate::COMMAND_LEN].try_into().unwrap())
                as usize;

        // read until all the data from the command has been recieved
        let _ = self
            .read_exact(&mut buffer[..command_len])
            .map_err(|_| crate::CommandError::RecieveFailed)?;

        crate::Command::try_from(&buffer[..command_len])
    }

    fn send(&self) {
        todo!()
    }
}
