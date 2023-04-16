#![deny(warnings)]

use core::convert::Infallible;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use std::net::{IpAddr, SocketAddr};

static INSTANCE_IP: &str = "http://0.0.0.0:8079";

#[tokio::main]
async fn main() {
    let default_ip = "0.0.0.0:8080";
    let addr: SocketAddr = default_ip.parse().unwrap();

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error :{e}");
    }
}

async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match hyper_reverse_proxy::call(client_ip, INSTANCE_IP, req).await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()),
    }
}
