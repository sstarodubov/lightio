use crate::file_storage::FileStorage;
use crate::http;
use crate::http::{HttpMethod, HttpReq};
use std::cell::RefCell;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::ops::{Deref};
use std::path::{Path};
use std::rc::Rc;

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq, tcp_stream: Rc<RefCell<&TcpStream>>);
    fn path(&self) -> &str;
    fn method(&self) -> HttpMethod;
}

const BUCKET_PATH: &str = "/bucket";
const OBJECT_PATH: &str = "/object";

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
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>)  {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if let Err(e) = self.file_storage.create_bucket(Path::new(bucket_name)) {
                    eprintln!("Failed to create bucket {}: {:?}", bucket_name, e);
                    output
                        .borrow_mut()
                        .write_all(http::SERVER_ERROR.as_bytes())
                        .unwrap()
                } else {
                    output
                        .borrow_mut()
                        .write_all(http::OK_RESPONSE.as_bytes())
                        .unwrap()
                }
            }
            None => output
                .borrow_mut()
                .write_all(http::BAD_REQUEST.as_bytes())
                .unwrap(),
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
                    output
                        .borrow_mut()
                        .write_all(http::SERVER_ERROR.as_bytes())
                        .unwrap();
                } else {
                    output
                        .borrow_mut()
                        .write_all(http::OK_RESPONSE.as_bytes())
                        .unwrap();
                }
            }
            None => {
                output
                    .borrow_mut()
                    .write_all(http::OK_RESPONSE.as_bytes())
                    .unwrap();
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

// read object
pub struct ReadObjectHandler {
    file_storage: &'static FileStorage,
}
impl ReadObjectHandler {
    pub fn new(file_storage: &'static FileStorage) -> Self {
        ReadObjectHandler { file_storage }
    }
}

impl ReadObjectHandler {
    fn file_size(file: &File) -> u64 {
        file.metadata().expect("file must be exist").len()
    }
}

impl HttpHandler for ReadObjectHandler {
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>) {
        let query_params = &req.query_params;
        let object_name = query_params.get("object_name");
        let bucket_name = query_params.get("bucket_name");
        let mut output = output.borrow_mut();
        if object_name.is_none() || bucket_name.is_none() {
            println!("object_name and bucket_name are required");
            output
                .write_all(http::BAD_REQUEST.as_bytes())
                .unwrap_or_else(|e| {
                    println!("cannot write response: {}", e);
                });
        } else {
            let bucket_name = bucket_name.unwrap();
            let object_name = object_name.unwrap();
            let obj_result = self
                .file_storage
                .open_file(Path::new(bucket_name).join(object_name).deref());
            if obj_result.is_err() {
                println!(
                    "object_name does not exist: {}, {}",
                    bucket_name,
                    &obj_result.unwrap_err()
                );
                output
                    .write_all(http::NOT_FOUND.as_bytes())
                    .expect("file is not found write panic");
                return;
            }

            let mut obj = obj_result.unwrap();
            let len = Self::file_size(&obj);
            output.write_all("HTTP/1.1 200 OK\r\n".as_bytes()).unwrap_or_else(|e| {
                println!("cannot write response status: {}", e);
            });
            output.write_all("Content-type: application/octet-stream\r\n".as_bytes()).unwrap_or_else(|e| {
                println!("cannot write content-type: {}", e);
            });
            let content_len = format!("Content-length: {}\r\n\r\n", len);
            output.write_all(content_len.as_bytes()).unwrap();
            let mut buff = [0; 1024 * 1024];
            loop {
                let read_bytes = obj.read(&mut buff).expect("read file panic");
                if read_bytes == 0 {
                    break;
                } else {
                    output.write_all(&buff[0..read_bytes]).unwrap();
                }
            }
        }
    }

    fn path(&self) -> &str {
        OBJECT_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::GET
    }
}

// create object
pub struct CreateObjectHandler {
    file_storage: &'static FileStorage,
}
impl CreateObjectHandler {
    pub fn new(file_storage: &'static FileStorage) -> Self {
        CreateObjectHandler { file_storage }
    }
}

impl HttpHandler for CreateObjectHandler {
    fn handle_request(&self, req: &mut HttpReq, output: Rc<RefCell<&TcpStream>>) {
        let mut output = output.borrow_mut();
        let query_params = &req.query_params;
        let bucket_name = query_params.get("bucket_name");
        let object_name = query_params.get("object_name");
        if bucket_name.is_none() || object_name.is_none() {
            println!("object_name and bucket_name are required");
            output.write_all(http::BAD_REQUEST.as_bytes()).expect("object name does not exist panic");
            return;
        }
        let bucket_name = bucket_name.unwrap();
        let object_name = object_name.unwrap();

        let size = req.headers.get("content-length");
        if size.is_none() {
            println!("content-length header missing");
            output.write_all(http::BAD_REQUEST.as_bytes()).expect("content length header missing panic");
            return;
        }

        let size = size.unwrap().trim().parse::<usize>();
        if size.is_err() {
            println!("content length value is not correct");
            output.write_all(http::BAD_REQUEST.as_bytes()).expect("content length value invalid");
            return;
        }
        let content_size = size.unwrap();
        let create_object_path = Path::new(bucket_name).join(object_name);
        let new_file = self.file_storage.create_file(create_object_path.as_path());
        match new_file {
            Ok(mut file) => {
                let mut buff = [0; 1024*1024];
                let mut cur_size: usize = 0;
                let body = &mut req.body;
                loop {
                    match body.read(&mut buff) {
                        Ok(0) => break,
                        Ok(written_size) => {
                            println!("{}", String::from_utf8_lossy(&buff[0..written_size]));
                            file.write_all(&buff[0..written_size]).unwrap_or_else(|e| {
                                println!("cannot write file: {}", e);
                            });
                            cur_size += written_size;
                            if cur_size >= content_size {
                                break;
                            }
                        }
                        Err(e) => {
                            println!("cannot read response: {}", e);
                        }
                    }
                }

                output.write_all(http::OK_RESPONSE.as_bytes()).expect("write response panic");
            }
            Err(e) => {
                println!("cannot create object file: {}", e);
                output.write_all(http::SERVER_ERROR.as_bytes()).expect("write response panic");
            }
        }
    }

    fn path(&self) -> &str {
        OBJECT_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::POST
    }
}
