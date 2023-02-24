#![deny(warnings)]

use std::env;

use hyper::{
    rt::{self, Future},
    service::service_fn_ok,
    Body, Response, Server,
};
extern crate hyper;
extern crate pretty_env_logger;

const DEFAULT_PORT: u16 = 8080;

fn run_server(port: u16) {
    pretty_env_logger::init();

    let addr = ([0, 0, 0, 0], port).into();

    let service = move || {
        service_fn_ok(move |_| {
            let hello = [
                format!("[{}]:", port),
                "hello".to_owned(),
                env::var("TARGET").unwrap_or_else(|_| "Rust demo!".to_owned()),
            ]
            .join(" ")
                + "\n";
            Response::new(Body::from(hello))
        })
    };

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| eprintln!("server error: {e}"));

    rt::run(server);
}

fn main() {
    run_server(DEFAULT_PORT)
}
