use command_handler::{CommandHandler, tcp_command_handler::TcpCommandHandler};

fn main() {
    // first create a command handler
    let command_handler = TcpCommandHandler::default();

    // loop forever for incomming connections
    loop {
        let connetion = command_handler.listen();
    }
}
