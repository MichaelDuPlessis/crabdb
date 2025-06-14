use logging::{debug, error, init_logger};
use server::{Connection, Response, Server, tcp_server::TcpServer};
use storage::{Storage, in_memory_store::InMemoryStore};

/// The engine of the database. It controls all the logic
struct Engine<S: Server, D: Storage> {
    server: S,
    storage: D,
}

impl<S: Server, D: Storage> Engine<S, D> {
    /// Creates a new Engine
    fn new(server: S, storage: D) -> Self {
        Self { server, storage }
    }

    /// Starts the server
    fn start(&mut self) -> Result<(), ()> {
        loop {
            // waiting for a connection
            debug!("Waiting for connection");
            let mut connection = self.server.listen();
            debug!("Connection recieved");

            // Now that a connection has been made continously recieve data
            loop {
                // recieving data
                let request = match connection.receive() {
                    Ok(request) => request,
                    Err(e) => {
                        error!("An error occured when recieving a request: {e:?}");
                        break;
                    }
                };

                // seeing what kind of request is made
                debug!("Sending response");
                let response = match request {
                    server::Request::Get(key) => self.storage.get(key),
                    server::Request::Set(key, object) => self.storage.set(key, object),
                    server::Request::Terminated => break,
                };

                let response = match response {
                    Ok(object) => Response::Payload(object),
                    Err(_) => Response::Error,
                };

                if let Err(e) = connection.send(response) {
                    error!("An error occured when sending a response: {e:?}");
                    break;
                }
            }
        }
    }
}

fn main() {
    // intializig logger
    init_logger(logging::LogLevel::Trace);

    let server = TcpServer::default();
    let storage = InMemoryStore::default();
    let mut engine = Engine::new(server, storage);

    engine.start().unwrap();
}
