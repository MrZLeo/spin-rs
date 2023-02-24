#![deny(warnings)]

use hyper::{
    rt::{self, Future},
    service::service_fn_ok,
    Body, Response, Server,
};
extern crate hyper;

const DEFAULT_PORT: u16 = 8080;

fn run_server(port: u16) {
    let addr = ([0, 0, 0, 0], port).into();

    let service = move || service_fn_ok(move |_| Response::new(Body::from("spin-rs")));

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| eprintln!("server error: {e}"));

    rt::run(server);
}

fn main() {
    run_server(DEFAULT_PORT)
}
