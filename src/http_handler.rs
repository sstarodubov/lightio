use crate::http::{HttpMethod, HttpReq};
use std::io::Read;

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq) -> u16;
    fn path(&self) -> String;
    fn method(&self) -> HttpMethod;
}

//for tests
pub struct HelloHandler;

unsafe impl Sync for HelloHandler {}
unsafe impl Send for HelloHandler {}

impl HttpHandler for HelloHandler {
    fn handle_request(&self, request: &mut HttpReq) -> u16 {
        let mut buff = [0; 6];
        let reader = &mut request.body;
        let result = reader.read(&mut buff).expect("Failed to read line");
        println!("{}", String::from_utf8_lossy(&buff[0..result]));
        let result = reader.read(&mut buff).expect("Failed to read line");
        println!("{}", String::from_utf8_lossy(&buff[0..result]));
        200
    }

    fn path(&self) -> String {
        String::from("/hello")
    }

    fn method(&self) -> HttpMethod {
        HttpMethod::GET
    }
}
