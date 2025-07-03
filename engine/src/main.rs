use logging::{error, info};
use server::{Command, Server};
use storage::{InMemoryStore, Store};

/// The port to listen on
const PORT: u16 = 4871;

fn main() {
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

        loop {
            let command = match connection.recieve() {
                Ok(command) => command,
                Err(e) => {
                    error!("Error recieving command: {e}");
                    break;
                }
            };

            let object = match command {
                Command::Get(key) => storage.retrieve(key),
                Command::Set(key, object) => storage.store(key, object),
            };

            if let Err(e) = connection.send(object) {
                error!("Error sending command: {e}");
                break;
            };
        }
    }
}
