use command_handler::{CommandHandler, Connection, tcp_command_handler::TcpCommandHandler};
use threadpool::ThreadPool;

fn main() {
    // instantiating the threadpool
    let mut threadpool = ThreadPool::default();

    // first create a command handler
    let command_handler = TcpCommandHandler::default();

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
