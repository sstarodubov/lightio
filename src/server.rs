use crate::http;
use crate::http::HttpReq;
use http::HttpMethod;
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str::Split;

const BUFFER_SIZE: usize = 64 * 1024;

pub struct HttpServer {
    port: u16,
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
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

    fn dispatch(mut stream: TcpStream) {
        let mut buff = [0; BUFFER_SIZE];
        loop {
            let result = stream.read(&mut buff);
            match result {
                Ok(bytes_count) => {
                    if bytes_count == 0 {
                        println!("Client closed connection");
                        break;
                    }
                    let request_len = http::find_request_len(&buff);
                    let request = http::extract_http_req(&buff, request_len);
                    match request {
                        Some(request) => {
                            println!("{:?}", request);
                            Self::write(&mut stream, http::OK);        
                        }
                        None => {
                            Self::write(&mut stream, http::BAD_REQUEST);
                            break;    
                        }
                    }
                }
                Err(e) => eprintln!("Error reading from stream: {}", e),
            }
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
}
