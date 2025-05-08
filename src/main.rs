mod server;
mod http;
mod http_handler;

use server::HttpServer;
fn main() {
    let server = HttpServer::new(8080);
    server.start();
    println!("Main");
}
