mod file_storage;
mod http;
mod http_handler;
mod server;
mod thread_pool;
mod http_client;

use crate::file_storage::FileStorageConfig;
use crate::http_handler::*;
use crate::server::HttpServerConfig;
use file_storage::FileStorage;
use server::HttpServer;
use crate::http_client::HttpClient;

fn main() {
    /*
    let file_storage_config = FileStorageConfig::new();
    let file_storage = Box::new(FileStorage::new(file_storage_config).unwrap());
    let file_storage: &'static FileStorage = Box::leak(file_storage);
    HttpServer::start(HttpServerConfig::new().handlers(vec![
        Box::new(BucketCreateHandler::new(file_storage)),
        Box::new(BucketDeleteHandler::new(file_storage)),
        Box::new(BucketExistsHandler::new(file_storage)),
        Box::new(ReadObjectHandler::new(file_storage)),
        Box::new(CreateObjectHandler::new(file_storage)),
    ]))
     */
    let req = HttpClient::new().get("http://localhost:8080/hello");
    req.send();
}
