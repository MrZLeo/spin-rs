#![deny(warnings)]

use core::convert::Infallible;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use std::{
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{process::Command, sync::OnceCell};

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

static INSTANCE_ADDR: &str = "http://0.0.0.0:8079";
static INSTANCE_START: OnceCell<()> = OnceCell::const_new();

/// Send the request to the real instance
async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // check whether instance exsit
    // Safety: make sure only one instance can start and only after instance start,
    // request can be response
    let _ = INSTANCE_START
        .get_or_init(|| async move {
            let start_time = SystemTime::now();
            println!(
                "Init instance at: {}",
                start_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
            );

            tokio::spawn(async move {
                Command::new("python3")
                    .arg("daemon.py")
                    .spawn()
                    .expect("fail to boot instance")
            })
            .await
            .unwrap();

            // yes, it is a magic number
            // I don't know how to decide the time to wait for instance start,
            // but I know `MAX_TRY` works
            const MAX_TRY: usize = 10000;
            for i in 0..=MAX_TRY {
                if let Ok(_) =
                    hyper_reverse_proxy::call(client_ip, INSTANCE_ADDR, Request::new(Body::empty()))
                        .await
                {
                    break;
                }
                if i == MAX_TRY {
                    panic!("Unable to boot instance function")
                }
            }
        })
        .await;

    match hyper_reverse_proxy::call(client_ip, INSTANCE_ADDR, req).await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()),
    }
}
