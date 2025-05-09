mod server;
mod http;
mod http_handler;

use server::HttpServer;
use crate::http_handler::HelloHandler;

fn main() {
    
    let mut server = HttpServer::new(8080);
    server.add_handler(HelloHandler);
    server.start();
    println!("Main");
}
