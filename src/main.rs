#![deny(warnings)]

use core::convert::Infallible;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use std::{
    net::{IpAddr, SocketAddr},
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::process::Command;

static INSTANCE_ADDR: &str = "http://0.0.0.0:8079";

#[tokio::main]
async fn main() {
    let default_addr = "0.0.0.0:8080";
    let addr: SocketAddr = default_addr.parse().unwrap();

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    if let Err(e) = server.await {
        eprintln!("server error :{e}");
    }
}

static INSTANCE_BOOT: AtomicBool = AtomicBool::new(false);

/// Send the request to the real instance
async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // check whether instance exsit
    if !INSTANCE_BOOT.load(Ordering::Relaxed) {
        tokio::spawn(async move {
            Command::new("python3")
                .arg("/daemon.py")
                .spawn()
                .expect("fail to boot instance")
        })
        .await
        .unwrap();
        INSTANCE_BOOT.store(true.into(), Ordering::Relaxed);
    }

    match hyper_reverse_proxy::call(client_ip, INSTANCE_ADDR, req).await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()),
    }
}
