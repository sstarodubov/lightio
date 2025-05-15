use crate::http::HttpMethod;
use std::collections::HashMap;
use std::io::ErrorKind::InvalidInput;
use std::io::{Error, Write};
use std::net::TcpStream;

pub struct HttpClient;

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            url: url.to_string(),
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }
}

pub struct Response;
pub struct RequestBuilder {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl RequestBuilder {
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, data: &str) -> Self {
        self.body = data.as_bytes().to_vec();
        self
    }

    pub fn method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

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
           conn.write_all(format!("Content-Length: {}\r\n", &self.body.len()).as_bytes())?; 
           conn.write_all(&self.body)?;
        }
        conn.write_all("\r\n".as_bytes())?;
        
        // recv response
        Ok(Response {})
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
