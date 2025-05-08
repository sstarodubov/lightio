use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

pub const HTTP_SPACE: &str = "\r\n";
pub const TEMPLATE_CLIENT_ERROR: &str = "HTTP/1.1 {} BAD REQUEST\r\nContent-Length: 0\r\n\r\n";
pub const TEMPLATE_OK: &str = "HTTP/1.1 {} OK\r\nContent-Length: 0\r\n\r\n";
pub const TEMPLATE_SERVER_ERROR: &str = "HTTP/1.1 {} INTERNAL ERROR\r\nContent-Length: 0\r\n\r\n";
pub const BAD_REQUEST: &[u8] = "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\n\r\n".as_bytes();
pub const SERVER_ERROR: &[u8] = "HTTP/1.1 500 INTERNAL ERROR\r\nContent-Length: 0\r\n\r\n".as_bytes();

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

pub fn parse_start_line(line: &str) -> Option<(HttpMethod, String)> {
    let parts = line.split_whitespace().collect::<Vec<&str>>();
    match parts[..] {
        [method, path, ..] => Some((HttpMethod::from_str(method), path.to_string())),
        _ => None,
    }
}
pub fn parse_headers(reader: &mut BufReader<&TcpStream>) -> Option<HashMap<String, String>> {
    let mut headers = HashMap::<String, String>::new();
    loop {
        let mut line = String::new();
        if let Err(e) = reader.read_line(&mut line) {
            eprintln!("http request read header error: {}", e);
            return None;
        }
        if line == "\r\n" || line.trim().is_empty() {
            println!("consumed request");
            return Some(headers);
        }
        match line.split_once(":") {
            Some((header, value)) => {
                headers.insert(header.to_string(), value.to_string());
            }
            None if line.trim().is_empty() => {
                //do nothing
            }
            None => {
                eprintln!("not valid header: {}", line);
            }
        };
    }
} 
