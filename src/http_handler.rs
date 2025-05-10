use crate::http::{HttpMethod, HttpReq};

pub trait HttpHandler {
    fn handle_request(&self, req: &mut HttpReq) -> u16;
    fn path(&self) -> String;
    fn method(&self) -> HttpMethod;
}
