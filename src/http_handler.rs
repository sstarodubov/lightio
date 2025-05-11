use crate::file_storage::{FileStorage};
use crate::http::{HttpMethod, HttpReq};
use std::path::Path;

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq) -> u16;
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
    fn handle_request(&self, req: &mut HttpReq) -> u16 {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if let Err(e) = self.file_storage.create_bucket(Path::new(bucket_name)) {
                    eprintln!("Failed to create bucket {}: {:?}", bucket_name, e);
                    500
                } else {
                    200
                }
            }
            None => 400,
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
    fn handle_request(&self, req: &mut HttpReq) -> u16 {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if let Err(e) = self.file_storage.delete_bucket(Path::new(bucket_name)) {
                    eprintln!("Failed to delete bucket {}: {:?}", bucket_name, e);
                    500
                } else {
                    200
                }
            }
            None => 200,
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
    fn handle_request(&self, req: &mut HttpReq) -> u16 {
        let query_params = &req.query_params;
        match query_params.get("bucket_name") {
            Some(bucket_name) => {
                if self.file_storage.bucket_exists(Path::new(bucket_name)) {
                    200
                } else {
                    404
                }
            }
            None => 400,
        }
    }
    fn path(&self) -> &str {
        BUCKET_PATH
    }
    fn method(&self) -> HttpMethod {
        HttpMethod::GET
    }
}
