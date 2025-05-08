use std::collections::HashMap;
use std::io::BufReader;
use std::net::TcpStream;

pub const HTTP_SPACE: &str = "\r\n";
pub const BAD_REQUEST_TEMPLATE: &str = "HTTP/1.1 {} BAD REQUEST\r\nContent-Length: 0\r\n\r\n";
pub const OK_TEMPLATE: &str = "HTTP/1.1 {} OK\r\nContent-Length: 0\r\n\r\n";
pub const INTERNAL_ERROR_TEMPLATE: &str = "HTTP/1.1 {} INTERNAL ERROR\r\nContent-Length: 0\r\n\r\n";

#[derive(Debug)]
pub enum HttpMethod {
    POST,
    GET,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> HttpMethod {
        match method {
            "POST" => HttpMethod::POST,
            "GET" => HttpMethod::GET,
            _ => HttpMethod::GET,
        }
    }
}

#[derive(Debug)]
pub struct HttpReq<'a> {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: BufReader<&'a TcpStream>,
}

pub fn handle_start_line(line: &str) -> Option<(HttpMethod, String)> {
    let parts = line.split_whitespace().collect::<Vec<&str>>();
    match parts[..] {
        [method, path, ..] => Some((HttpMethod::from_str(method), path.to_string())),
        _ => None,
    }
}
