use crate::http;
use crate::http::{HttpMethod, HttpReq};
use crate::http_handler::HttpHandler;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::Shutdown::Both;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

type BoxedHttpHandler = Box<dyn HttpHandler + Send + Sync>;
type HttpHandlerMap = HashMap<HttpMethod, HashMap<String, BoxedHttpHandler>>;

pub struct HttpServer;

pub struct HttpServerConfig {
    port: u16,
    handlers: Vec<BoxedHttpHandler>,
    max_req: i64,
}

impl HttpServerConfig {
    pub fn new() -> Self {
       Self {
           port: 8080,
           handlers: Vec::new(),
           max_req: -1
       }
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn handlers(mut self, handlers: Vec<BoxedHttpHandler>) -> Self {
        self.handlers = handlers;
        self
    }
    
    pub fn max_req(mut self, max_req: i64) -> Self {
        self.max_req = max_req;
        self
    }
}

impl HttpServer {

    pub fn start(config: HttpServerConfig) {
        let HttpServerConfig { handlers, port, max_req} = config;
        let handlers = Arc::new(Self::create_handler_map(handlers));
        let mut handled_requests = 0;
        let listener =
            TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind port");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler_map = Arc::clone(&handlers);
                    thread::spawn(move || Self::dispatch(stream, handler_map));
                }
                Err(e) => eprintln!("Http request e: {}", e),
            }
            handled_requests += 1;
            if max_req != -1 && handled_requests >= max_req {
                println!("stop server");
                return;
            }
        }
    }

    pub fn start_on_thread(config: HttpServerConfig) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            Self::start(config);
        })
    }

    fn create_handler_map(handlers: Vec<BoxedHttpHandler>) -> HttpHandlerMap {
        let mut map: HttpHandlerMap = HashMap::new();
        for handler in handlers {
            if !map.contains_key(&handler.method()) {
                map.insert(handler.method(), HashMap::new());
            }

            let path_hm = map.get_mut(&handler.method()).expect("No handler");
            path_hm.insert(handler.path(), handler);
        }

        map
    }

    fn dispatch(mut stream: TcpStream, handlers: Arc<HttpHandlerMap>) {
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
                query_params: HashMap::new(), //todo()
            };

            let method_hm = handlers.get(&request.method);
            if method_hm.is_none() {
                println!( "method not found warning. method: {:?}", request.method);
                Self::write(&mut stream, http::NOT_FOUND);
                continue;
            }
            let method_hm = method_hm.expect("method extracting panic");
            let path = &request.path;
            let http_handler = method_hm.get(path);
            if http_handler.is_none() {
                println!(
                    "handler get warning. path: {}, method: {:?}",
                    request.path, request.method
                );
                Self::write(&mut stream, http::NOT_FOUND);
                continue;
            }

            let handler = http_handler.expect("handler is not found");
            let (template, code) = match handler.handle_request(&mut request) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {}
}
