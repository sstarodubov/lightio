use crate::http::HttpMethod;
use std::collections::HashMap;
use std::io::ErrorKind::InvalidInput;
use std::io::{Error, Read, Write};

use std::net::TcpStream;

#[allow(dead_code)]
pub struct HttpClient;

impl HttpClient {

    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            url: url.to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

   
    #[allow(dead_code)]
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            url: url.to_string(),
            method: HttpMethod::POST,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
}

#[allow(dead_code)]
pub struct Response {
    status_code: u16,
}
impl Response {
    #[allow(dead_code)]
    pub fn new(status_code: u16) -> Self {
        Self { status_code }
    }

    #[allow(dead_code)]
    pub fn status(&self) -> u16 {
        self.status_code
    }
}

pub struct RequestBuilder {

    #[allow(dead_code)]
    url: String,
    method: HttpMethod,

    #[allow(dead_code)]
    headers: HashMap<String, String>,

    #[allow(dead_code)]
    body: Vec<u8>,
}

impl RequestBuilder {

    #[allow(dead_code)]
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn body(mut self, data: &str) -> Self {
        self.body = data.as_bytes().to_vec();
        self
    }

    #[allow(dead_code)]
    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    #[allow(dead_code)]
    pub fn send(&self) -> Result<Response, Error> {
        let url = Self::parse_url(&self.url);
        if url.is_none() {
            println!("not valid url: {}", &self.url);
            return Err(Error::new(InvalidInput, "not valid url"));
        }
        let (host, path) = url.unwrap();
        let mut conn = TcpStream::connect(&host)?;
        // send request
        conn.write_all(format!("{} {} HTTP/1.1\r\n", self.method.as_str(), path).as_bytes())?;
        for (header_key, header_val) in self.headers.iter() {
            conn.write_all(format!("{}: {}\r\n", header_key, header_val).as_bytes())?;
        }
        conn.write_all(format!("Host: {}\r\n", &host).as_bytes())?;
        if !&self.body.is_empty() {
            conn.write_all(format!("Content-Length: {}\r\n\r\n", &self.body.len()).as_bytes())?;
            conn.write_all(&self.body)?;
        } else {
            conn.write_all("\r\n".as_bytes())?;
        }

        let mut response = String::new();
        conn.read_to_string(&mut response)?;
        println!("{:?}", &response);
        let status = Self::extract_status(&response);
        match status {
            None => {
                println!("not valid status: {}", &response);
                Ok(Response::new(400))
            },
            Some(val) => 
                Ok(Response::new(val))
        } 
    }

    fn extract_status(response: &str) -> Option<u16> {
        let terms = response.trim().split(" ");
        for (i, term) in terms.enumerate() {
            if i == 1 {
                return term.parse::<u16>().map_or(None, |r| Some(r));
            }
        }
        None
    }

    fn parse_url(uri: &str) -> Option<(String, String)> {
        let (protocol, url) = uri.trim().split_once("//")?;
        if protocol == "http:" {
            let (host, path) = url.trim().split_once("/")?;
            Some((host.to_owned(), format!("/{}", path)))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test() {
        let (host, path) =
            RequestBuilder::parse_url("http://localhost:8084/hello?hello=world&test=1").unwrap();
        assert_eq!(host, "localhost:8084");
        assert_eq!(path, "/hello?hello=world&test=1");
    }
}
