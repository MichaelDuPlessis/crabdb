use server::{Connection, Server, tcp_server::TcpServer};
use storage::Storage;
use threadpool::ThreadPool;

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
            let mut connection = self.server.listen();

            // recieving data
        }
    }
}

fn main() {
    // instantiating the threadpool
    let mut threadpool = ThreadPool::default();

    // first create a command handler
    let command_handler = TcpServer::default();

    // loop forever for incomming connections
    loop {
        let mut connection = command_handler.listen();

        // when a connection is recieved send it to the threadpool
        threadpool.execute(move || {
            loop {
                let command = match connection.recieve() {
                    Ok(command) => command,
                    Err(err) => {
                        println!("An error occured: {:?}", err);
                        break;
                    }
                };

                println!("Command recieved: {:?}", command);
            }
        });
    }
}
