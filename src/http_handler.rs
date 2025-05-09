use crate::http::{HttpMethod, HttpReq};
use std::io::Read;

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq) -> u16;
    fn path(&self) -> String;
    fn method(&self) -> HttpMethod;
}
