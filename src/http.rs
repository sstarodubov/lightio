use std::collections::HashMap;
use std::str::Split;

pub const HTTP_SPACE: &str = "\r\n";
pub const BAD_REQUEST: &[u8] = "HTTP/1.1 400 BAD REQUEST\r\nContent-Length: 0\r\n\r\n".as_bytes();
pub const OK: &[u8] = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".as_bytes();

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
pub struct HttpReq {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
}

pub fn find_request_len(buff: &[u8]) -> usize {
    for i in 0..buff.len() - 4 {
        if buff[i] == 13u8 && buff[i + 1] == 10u8 && buff[i + 2] == 13u8 && buff[i + 3] == 10u8 {
            return i + 4;
        }
    }
    buff.len()
}

pub fn extract_http_req(buff: &[u8], len: usize) -> Option<HttpReq> {
    let req_buff = &buff[..len];
    let req = String::from_utf8(req_buff.to_vec());
    if let Ok(req) = req {
        let mut req = req.split(HTTP_SPACE);
        let start_line = req.next()?;
        let parts = start_line.split_whitespace().collect::<Vec<&str>>();
        match parts[..] {
            [method, path, ..] => {
                println!("Метод: {}, Путь: {}", method, path);
                Some(HttpReq {
                    method: HttpMethod::from_str(method),
                    path: path.to_string(),
                    headers: extract_headers(req),
                })
            }
            _ => {
                eprintln!("not valid start line. {}", start_line);
                None
            }
        }
    } else {
        eprintln!("utf8 request convert error: {:?}", req_buff);
        None
    }
}

pub fn extract_headers(req: Split<&str>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in req {
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
        }
    }
    headers
}
