use logging::{error, info, init_logger, trace};
use server::{Command, Server};
use storage::{InMemoryStore, Store};

/// The port to listen on
const PORT: u16 = 7227;

fn main() {
    // initializing logger
    init_logger(logging::LogLevel::Trace);

    info!("Starting server");

    // the server to listen on
    let server = Server::new(PORT);

    // used to store the objects
    let mut storage = InMemoryStore::default();

    info!("Listening on port: {PORT}");
    loop {
        let Ok(mut connection) = server.listen() else {
            error!("Something bad happend");
            break;
        };
        info!("Connection recieved");

        loop {
            let command = match connection.recieve() {
                Ok(command) => command,
                Err(e) => {
                    error!("Error recieving command: {e}");
                    break;
                }
            };
            trace!("Command recieved: {:?}", command);

            let object = match command {
                Command::Get(key) => storage.retrieve(key),
                Command::Set(key, object) => storage.store(key, object),
            };

            trace!("Sending response: {:?}", object);
            if let Err(e) = connection.send(object) {
                error!("Error sending command: {e}");
                break;
            };
        }
    }
}
