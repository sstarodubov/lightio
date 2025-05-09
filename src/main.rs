mod http;
mod http_handler;
mod server;

use std::io::Write;
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;
use crate::server::HttpServerConfig;
use server::HttpServer;

fn main() {
    HttpServer::start_on_thread(HttpServerConfig::new().port(8080));
    thread::sleep(Duration::from_secs(1));
    let response = Client::new().get("http://localhost:8080/").send();
    // Выводим статус и тело
    println!("response: {:?}", response);
}
