mod http;
mod http_handler;
mod server;

use crate::http_handler::HelloHandler;
use crate::server::HttpServerConfig;
use server::HttpServer;

fn main() {
    let config = HttpServerConfig::new()
        .port(8080)
        .handlers(vec![Box::new(HelloHandler)]);
    
    HttpServer::start(config);
    println!("main stops")
}
