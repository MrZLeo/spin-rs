#![deny(warnings)]

extern crate tiny_http;
use tiny_http::{Response, Server};

const DEFAULT_IP: &str = "0.0.0.0:8080";
const RESPONSE: &str = "spin-rs";

fn main() {
    let server = Server::http(DEFAULT_IP).unwrap();
    for request in server.incoming_requests() {
        let response = Response::from_string(RESPONSE);
        request.respond(response).unwrap();
    }
}
