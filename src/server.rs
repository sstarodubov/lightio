use crate::http;
use crate::http::HttpReq;
use crate::http_handler::HttpHandler;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown::Both;
use std::net::{TcpListener, TcpStream};

const BUFFER_SIZE: usize = 6;

pub struct HttpServer {
    port: u16,
    handlers: Vec<Box<dyn HttpHandler>>,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            handlers: vec![],
        }
    }

    pub fn start(&self) {
        let listener =
            TcpListener::bind(format!("127.0.0.1:{}", self.port)).expect("Failed to bind port");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => Self::dispatch(stream),
                Err(e) => eprintln!("Http request e: {}", e),
            }
        }
    }
    
    pub fn add_handler(handler: impl HttpHandler) {
       todo!() 
    }

    fn dispatch(mut stream: TcpStream) {
        loop {
            let mut reader = BufReader::new(&stream);
            let mut start_line = String::new();
            if let Err(e) = reader.read_line(&mut start_line) {
                eprintln!("start line read error: {}", e);
                Self::write_and_close(&mut stream, http::SERVER_ERROR);
                break;
            }
            if start_line.is_empty() {
                println!("connection closed");
                break;
            }
            let start_line = http::parse_start_line(&start_line);
            if start_line.is_none() {
                println!("convert start line error");
                Self::write_and_close(&mut stream, http::BAD_REQUEST);
                break;
            }

            let Some((method, path)) = start_line else {
                panic!(
                    "method and path extracting panic. start line {:?}",
                    start_line
                )
            };

            let headers = http::parse_headers(&mut reader);
            if headers.is_none() {
                println!("headers parsing error");
                Self::write_and_close(&mut stream, http::BAD_REQUEST);
                break;
            }

            let mut request = HttpReq {
                path,
                method,
                headers: headers.unwrap(),
                body: reader,
            };

            let (template, code) = match handle_req(&mut request) {
                code @ 200..300 => (http::TEMPLATE_OK, code),
                code @ 400..499 => (http::TEMPLATE_CLIENT_ERROR, code),
                code => (http::TEMPLATE_SERVER_ERROR, code),
            };

            let binding = template.replace("{}", &code.to_string());
            let response = binding.as_bytes();
            Self::write(&mut stream, response);
        }
    }

    fn write(tcp_stream: &mut TcpStream, msg: &[u8]) {
        if let Err(e) = tcp_stream.write(msg) {
            eprintln!("Error writing to stream: {}, msg {:?}", e, msg);
            return;
        }
        if let Err(e) = tcp_stream.flush() {
            eprintln!("Error flushing stream: {}, msg {:?}", e, msg);
        }
    }

    fn write_and_close(tcp_stream: &mut TcpStream, msg: &[u8]) {
        Self::write(tcp_stream, msg);
        tcp_stream
            .shutdown(Both)
            .unwrap_or_else(|e| eprintln!("Error shutting down stream: {}, msg {:?}", e, msg))
    }
}

// test function
fn handle_req(request: &mut HttpReq) -> u16 {
    let mut buff = [0; BUFFER_SIZE];
    let reader = &mut request.body;
    let result = reader.read(&mut buff).expect("Failed to read line");
    println!("{}", String::from_utf8_lossy(&buff[0..result]));
    let result = reader.read(&mut buff).expect("Failed to read line");
    println!("{}", String::from_utf8_lossy(&buff[0..result]));
    200
}
