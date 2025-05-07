mod server;
mod http;

use server::HttpServer;
fn main() {
    let server = HttpServer::new(8080);
    server.start();
    println!("Main");
}
