use crate::http;
use crate::http::HttpReq;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

const BUFFER_SIZE: usize = 6;

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
        let mut reader = BufReader::new(&stream);
        let mut start_line = String::new();
        if let Err(e) = reader.read_line(&mut start_line) {
            eprintln!("start line read error: {}", e);
            return;
        }

        let start_line = http::handle_start_line(&start_line);
        if start_line.is_none() {
            eprintln!("convert start line error");
            return;
        }

        let Some((method, path)) = start_line else {
            panic!(
                "method and path extracting panic. start line {:?}",
                start_line
            )
        };
        let mut headers = HashMap::<String, String>::new();
        loop {
            let mut line = String::new();
            if let Err(e) = reader.read_line(&mut line) {
                eprintln!("http request read header error: {}", e);
                break;
            }
            if line == "\r\n" || line.trim().is_empty() {
                println!("consumed request");
                break;
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
        let mut request = HttpReq {
            path,
            method,
            headers,
            body: reader,
        };
        let (template, code) = match handle_req(&mut request) {
            code @ 200..300 => (http::OK_TEMPLATE, code),
            code @ 400..499 => (http::BAD_REQUEST_TEMPLATE, code),
            code => (http::INTERNAL_ERROR_TEMPLATE, code),
        };

        let binding = template.replace("{}", &code.to_string());
        let response = binding.as_bytes();
        Self::write(&mut stream, response);
        if let Err(e) = stream.shutdown(Shutdown::Both) {
            eprintln!("http request shutdown error: {}", e);
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

fn handle_req(request: &mut HttpReq) -> u16 {
    let mut buff = [0; BUFFER_SIZE];
    let reader = &mut request.body;
    let result = reader.read(&mut buff).expect("Failed to read line");
    println!("{}", String::from_utf8_lossy(&buff[0..result]));
    let result = reader.read(&mut buff).expect("Failed to read line");
    println!("{}", String::from_utf8_lossy(&buff[0..result]));
    200
}
