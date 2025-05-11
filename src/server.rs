use crate::http;
use crate::thread_pool;
use crate::http::{HttpMethod, HttpReq};
use crate::http_handler::HttpNoBodyHandler;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::Shutdown::Both;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

type NoBodyHttpHandler = Box<dyn HttpNoBodyHandler + Send + Sync>;
type NoBodyHttpHandlerMap = HashMap<HttpMethod, HashMap<String, NoBodyHttpHandler>>;

pub struct HttpServer;

pub struct HttpServerConfig {
    port: u16,
    handlers: Vec<NoBodyHttpHandler>,
    max_req: i64,
    pool_size: usize,
}

impl HttpServerConfig {
    pub fn new() -> Self {
        Self {
            port: 8080,
            handlers: Vec::new(),
            max_req: -1,
            pool_size: 4,
        }
    }
    
    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool_size = size;
        self
    }
    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn handlers(mut self, handlers: Vec<NoBodyHttpHandler>) -> Self {
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
        let HttpServerConfig {
            handlers,
            port,
            max_req,
            pool_size
        } = config;

        let pool = thread_pool::ThreadPool::new(pool_size).expect("thread pool create error"); 
        let handlers = Arc::new(Self::create_handler_map(handlers));
        let mut handled_requests = 0;
        let listener =
            TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to bind port");
        println!("Listening on {}", port);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler_map = Arc::clone(&handlers);
                    pool.execute(move || Self::dispatch(stream, handler_map));
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

    fn create_handler_map(handlers: Vec<NoBodyHttpHandler>) -> NoBodyHttpHandlerMap {
        let mut map: NoBodyHttpHandlerMap = HashMap::new();
        for handler in handlers {
            if !map.contains_key(&handler.method()) {
                map.insert(handler.method(), HashMap::new());
            }

            let path_hm = map.get_mut(&handler.method()).expect("No handler");
            path_hm.insert(handler.path().to_string(), handler);
        }

        map
    }

    fn dispatch(mut stream: TcpStream, handlers: Arc<NoBodyHttpHandlerMap>) {
        loop {
            let mut reader = BufReader::new(&stream);
            let mut start_line = String::new();
            if let Err(e) = reader.read_line(&mut start_line) {
                eprintln!("start line read error: {}", e);
                Self::write_and_close(&mut stream, http::SERVER_ERROR.as_bytes());
                break;
            }
            if start_line.is_empty() {
                println!("connection closed");
                break;
            }
            let start_line = http::parse_start_line(&start_line);
            if start_line.is_none() {
                println!("convert start line error");
                Self::write_and_close(&mut stream, http::BAD_REQUEST.as_bytes());
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
                Self::write_and_close(&mut stream, http::BAD_REQUEST.as_bytes());
                break;
            }
            let (path, query_params) = http::parse_query_params(path);
            let mut request = HttpReq {
                path,
                method,
                headers: headers.unwrap(),
                body: reader,
                query_params
            };

            let method_hm = handlers.get(&request.method);
            if method_hm.is_none() {
                println!("method not found warning. method: {:?}", request.method);
                Self::write(&mut stream, http::NOT_FOUND.as_bytes());
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
                Self::write(&mut stream, http::NOT_FOUND.as_bytes());
                continue;
            }
            
            let handler = http_handler.expect("handler is not found");
            let (template, code) = match handler.handle_request(&mut request) {
                code @ 200..300 => (http::TEMPLATE_OK, code),
                code @ 400..499 => (http::TEMPLATE_CLIENT_ERROR, code),
                code => (http::TEMPLATE_SERVER_ERROR, code),
            };
            
            
            let binding:String = match code {
                404 => http::NOT_FOUND.to_string(),   
                c => template.replace("{}", &c.to_string())
            };
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
    use reqwest::Method;
    use reqwest::blocking::Client;
    use reqwest::blocking::Response;
    use std::cmp::PartialEq;
    use std::time::Duration;

    struct TestHandler;
    unsafe impl Sync for TestHandler {}
    unsafe impl Send for TestHandler {}

    impl HttpNoBodyHandler for TestHandler {
        fn handle_request(&self, request: &mut HttpReq) -> u16 {
            if request.headers.contains_key("content-length") {
                let mut start_line = String::new();
                request.body.read_line(&mut start_line).expect("read body");
                if start_line == "helloworld\r\n" {
                    201
                } else {
                    400
                }
            } else if request.headers.contains_key("x-query") {
                let query_params = &request.query_params;
                let hello = query_params.get("hello");
                let test = query_params.get("test");
                if hello.is_none() || test.is_none() {
                    400
                } else {
                    let world = hello.unwrap();
                    let one = test.unwrap();
                    if world == "world" && one == "1" {
                        205
                    } else {
                        400
                    }
                }
            } else {
                200
            }
        }

        fn path(&self) -> &str {
            "/hello"
        }

        fn method(&self) -> HttpMethod {
            HttpMethod::GET
        }
    }

    fn start_server(port: u16, handler: impl HttpNoBodyHandler + Sync + Send + 'static) {
        HttpServer::start_on_thread(
            HttpServerConfig::new()
                .port(port)
                .max_req(1)
                .handlers(vec![Box::new(handler)]),
        );
        thread::sleep(Duration::from_millis(200));
    }

    fn send_req(port: u16, method: Method, path: &str) -> Response {
        let client = Client::new();
        let url = format!("http://localhost:{}/{}", port, path);
        let request = match method {
            Method::GET => client.get(&url),
            Method::POST => client.post(&url),
            _ => panic!("unknown method"),
        };
        request.send().unwrap()
    }

    #[test]
    fn send_fine_request() {
        start_server(8080, TestHandler);
        assert_eq!(200, send_req(8080, Method::GET, "hello").status().as_u16());
    }

    #[test]
    fn send_unknown_method() {
        start_server(8081, TestHandler);
        assert_eq!(404, send_req(8081, Method::POST, "hello").status().as_u16());
    }

    #[test]
    fn send_unknown_path() {
        start_server(8082, TestHandler);
        assert_eq!(404, send_req(8082, Method::GET, "hoho").status().as_u16());
    }

    #[test]
    fn send_body() {
        start_server(8083, TestHandler);

        let client = Client::new();
        let req = client
            .get("http://localhost:8083/hello")
            .header("content-length", "?")
            .body("helloworld\r\n");
        let response = req.send().unwrap();

        assert_eq!(201, response.status().as_u16());
    }

    #[test]
    fn send_query_params() {
        start_server(8084, TestHandler);

        let client = Client::new();
        let req = client
            .get("http://localhost:8084/hello?hello=world&test=1")
            .header("X-QUERY", "?");
        let response = req.send().unwrap();

        assert_eq!(205, response.status().as_u16());
    }
}
