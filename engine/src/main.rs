use logging::{error, info, init_logger, trace};
use server::{Command, Server};
use std::sync::Arc;
use storage::{Store, append_only_log::AppendOnlyLogStore, in_memory_store::InMemoryStore};
use threadpool::ThreadPool;

mod link_resolver;

/// The port to listen on
const PORT: u16 = 7227;

fn main() {
    // initializing logger
    init_logger(logging::LogLevel::Trace);

    info!("Starting server");

    // the server to listen on
    let server = Server::new(PORT);

    // used to store the objects with AOL recovery
    let storage = match AppendOnlyLogStore::new_with_recovery(
        "./data",
        unsafe { std::num::NonZeroUsize::new_unchecked(4) },
        InMemoryStore::new(4),
    ) {
        Ok(store) => Arc::new(store),
        Err(e) => {
            error!("Failed to initialize storage with recovery: {}", e);
            return;
        }
    };

    // Creating a threadpool
    let mut thread_pool = ThreadPool::default();

    info!("Listening on port: {PORT}");
    loop {
        let Ok(mut connection) = server.listen() else {
            error!("Something bad happend");
            break;
        };
        info!("Connection recieved");

        let storage = Arc::clone(&storage);
        thread_pool.execute(move || {
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
                    Command::Get(key, _) => storage.retrieve(key),
                    Command::Set(key, object) => storage.store(key, object),
                    Command::Delete(key) => storage.remove(key),
                    Command::Close => break,
                };

                match object {
                    Ok(object) => {
                        trace!("Sending response: {:?}", object);
                        if let Err(e) = connection.send(object) {
                            error!("Error sending command: {e}");
                        };
                    }
                    Err(e) => error!("Error occured saving object: {e:?}"),
                }
            }
        });
    }
}
