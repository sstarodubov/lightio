use crate::file_storage::FileStorage;
use crate::http;
use crate::http::{HttpMethod, HttpReq};
use std::cell::RefCell;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::rc::Rc;

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq, tcp_stream: Rc<RefCell<&TcpStream>>);
    fn path(&self) -> &str;
    fn method(&self) -> HttpMethod;
}

const BUCKET_PATH: &str = "/bucket";

// create bucket
pub struct BucketCreateHandler {
    file_storage: &'static FileStorage,
}
impl BucketCreateHandler {
    pub fn new(file_storage: &'static FileStorage) -> Self {
        BucketCreateHandler { file_storage }
    }
}
impl HttpHandler for BucketCreateHandler {
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>) {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if let Err(e) = self.file_storage.create_bucket(Path::new(bucket_name)) {
                    eprintln!("Failed to create bucket {}: {:?}", bucket_name, e);
                    output.borrow_mut().write_all(http::SERVER_ERROR.as_bytes()).unwrap() 
                } else {
                    output.borrow_mut().write_all(http::OK_RESPONSE.as_bytes()).unwrap()
                }
            }
            None => {
                output.borrow_mut().write_all(http::BAD_REQUEST.as_bytes()).unwrap()
            } ,
        }
    }
    fn path(&self) -> &str {
        BUCKET_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::POST
    }
}

//delete bucket
pub struct BucketDeleteHandler {
    file_storage: &'static FileStorage,
}
impl BucketDeleteHandler {
    pub fn new(file_storage: &'static FileStorage) -> Self {
        BucketDeleteHandler { file_storage }
    }
}
impl HttpHandler for BucketDeleteHandler {
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>) {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if let Err(e) = self.file_storage.delete_bucket(Path::new(bucket_name)) {
                    eprintln!("Failed to delete bucket {}: {:?}", bucket_name, e);
                    output.borrow_mut().write_all(http::SERVER_ERROR.as_bytes()).unwrap();
                } else {
                    output.borrow_mut().write_all(http::OK_RESPONSE.as_bytes()).unwrap();
                }
            }
            None => {
                output.borrow_mut().write_all(http::OK_RESPONSE.as_bytes()).unwrap();
            } 
        }
    }
    fn path(&self) -> &str {
        BUCKET_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::DELETE
    }
}

//exists bucket
pub struct BucketExistsHandler {
    file_storage: &'static FileStorage,
}
impl BucketExistsHandler {
    pub fn new(file_storage: &'static FileStorage) -> Self {
        BucketExistsHandler { file_storage }
    }
}
impl HttpHandler for BucketExistsHandler {
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>) {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if self.file_storage.bucket_exists(Path::new(bucket_name)) {
                    output
                        .borrow_mut()
                        .write_all(http::OK_RESPONSE.as_bytes())
                        .unwrap();
                } else {
                    output
                        .borrow_mut()
                        .write_all(http::NOT_FOUND.as_bytes())
                        .unwrap();
                }
            }
            None => {
                output
                    .borrow_mut()
                    .write_all(http::BAD_REQUEST.as_bytes())
                    .unwrap();
            }
        }
    }
    fn path(&self) -> &str {
        BUCKET_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::GET
    }
}
