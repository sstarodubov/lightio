mod http;
mod http_handler;
mod server;
mod thread_pool;

use std::thread;
use std::thread::Thread;
use std::time::Duration;
use reqwest::blocking::Client;
use crate::server::HttpServerConfig;
use server::HttpServer;

fn main() {
    thread_pool::ThreadPool::new(3);
}
