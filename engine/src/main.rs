use logging::{error, info};
use server::Server;

/// The port to listen on
const PORT: u16 = 4871;

fn main() {
    info!("Starting server");

    // The server to listen on
    let mut server = Server::new(PORT);

    info!("Listening on port: {PORT}");
    loop {
        let Ok(mut connection) = server.listen() else {
            error!("Something bad happend");
            break;
        };

        loop {
            let command = connection.recieve();
        }
    }
    
}
