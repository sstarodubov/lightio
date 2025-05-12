use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;

pub const TEMPLATE_CLIENT_ERROR: &str = "HTTP/1.1 {} BAD REQUEST\r\nContent-Length: 0\r\n\r\n";
pub const TEMPLATE_OK: &str = "HTTP/1.1 {} OK\r\nContent-Length: 0\r\n\r\n";

pub const TEMPLATE_SERVER_ERROR: &str = "HTTP/1.1 {} INTERNAL ERROR\r\nContent-Length: 0\r\n\r\n";
pub const BAD_REQUEST: &str = "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\n\r\n";
pub const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 0\r\n\r\n";
pub const SERVER_ERROR: &str =
    "HTTP/1.1 500 INTERNAL ERROR\r\nContent-Length: 0\r\n\r\n";
pub const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HttpMethod {
    POST,
    GET,
    DELETE,
}

impl HttpMethod {
    pub fn from_str(method: &str) -> HttpMethod {
        match method {
            "POST" => HttpMethod::POST,
            "GET" => HttpMethod::GET,
            "DELETE" => HttpMethod::DELETE,
            unknown => panic!("Unknown HTTP method: {}", unknown),
        }
    }
}

#[derive(Debug)]
pub struct HttpReq<'a> {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: BufReader<&'a TcpStream>,
    pub query_params: HashMap<String, String>,
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
                headers.insert(header.to_string().to_lowercase(), value.to_string());
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

pub fn parse_query_params(path: String) -> (String, HashMap<String, String>) {
    if let Some((path, params)) = path.split_once("?") {
        let mut params_map = HashMap::<String, String>::new();
        for key_val in params.split("&") {
            if let Some((key, val)) = key_val.split_once("=") {
                params_map.insert(key.to_string(), val.to_string());
            }
        }
        (path.to_string(), params_map)
    } else {
        (path.to_string(), HashMap::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_query_params_test() {
        let (path, params) = parse_query_params("/hello?hello=world&test=1".to_string());

        assert_eq!("/hello", path);
        assert_eq!(2, params.len());
        assert_eq!("world", params["hello"]);
        assert_eq!("1", params["test"]);
    }

    #[test]
    fn parse_query_params_test_4() {
        let (path, params) = parse_query_params("/hello?hello=world?&test=1".to_string());

        assert_eq!("/hello", path);
    }

    #[test]
    fn parse_query_params_test_3() {
        let (path, params) = parse_query_params("/hello?".to_string());

        assert_eq!("/hello", path);
        assert_eq!(0, params.len());
    }

    #[test]
    fn parse_query_params_test_2() {
        let (path, params) = parse_query_params("/hello".to_string());

        assert_eq!("/hello", path);
        assert_eq!(0, params.len());
    }
}
