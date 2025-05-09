mod http;
mod http_handler;
mod server;

use std::io::Write;
use std::thread;
use std::time::Duration;
use crate::server::HttpServerConfig;
use server::HttpServer;

fn main() {
    HttpServer::start_on_thread(HttpServerConfig::new().port(8080));
    thread::sleep(Duration::from_secs(1));
    let response = isahc::get("http://localhost:8080");
    // Выводим статус и тело
    println!("response: {:?}", response);
}
