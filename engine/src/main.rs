use std::sync::Arc;

use crab_core::{
    factory::{ObjectFactory, new_db_object, register_factory},
    object::Object,
    types::{int::Int, null::Null, text::Text},
};
use logging::{debug, error, init_logger};
use server::{Connection, Response, Server, tcp_server::TcpServer};
use storage::{Storage, StorageError, in_memory_store::InMemoryStore};
use threadpool::ThreadPool;

/// The engine of the database. It controls all the logic
struct Engine<S: Server, D: Storage> {
    server: S,
    storage: Arc<D>,
}

impl<S: Server, D: Storage + Send + Sync + 'static> Engine<S, D>
where
    S::Handler: 'static,
{
    /// Creates a new Engine
    fn new(server: S, storage: D) -> Self {
        Self {
            server,
            storage: Arc::new(storage),
        }
    }

    /// Starts the server
    fn start(&mut self) -> Result<(), ()> {
        // Creating the threadpool
        let mut thread_pool = ThreadPool::default();

        loop {
            // waiting for a connection
            debug!("Waiting for connection");
            let mut connection = self.server.listen();
            debug!("Connection recieved");

            // Now that a connection has been made continously recieve data
            // And spawn a new thread
            let storage = Arc::clone(&self.storage);
            thread_pool.execute(move || {
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
                        server::Request::Get(key) => storage.get(key),
                        server::Request::Set(key, object) => {
                            if let Ok(object) = new_db_object(object) {
                                storage
                                    .set(key, object)
                                    .map_err(|_| StorageError::SetFailed)
                            } else {
                                Err(StorageError::SetFailed)
                            }
                        }
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
            });
        }
    }
}

fn main() {
    // intializig logger
    init_logger(logging::LogLevel::Trace);

    // registring types
    register_factory(
        0,
        ObjectFactory::new(
            "null",
            Box::new(|_| Ok(Box::new(Null) as Box<dyn Object + Send + Sync>)),
        ),
    );
    register_factory(
        1,
        ObjectFactory::new("int", Box::new(Int::from_raw_object_data)),
    );
    register_factory(
        2,
        ObjectFactory::new("text", Box::new(Text::from_raw_object_data)),
    );

    let server = TcpServer::default();
    let storage = InMemoryStore::default();
    let mut engine = Engine::new(server, storage);

    engine.start().unwrap();
}
