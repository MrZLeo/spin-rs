#![deny(warnings)]

use core::convert::Infallible;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use std::{
    net::{IpAddr, SocketAddr},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{process::Command, sync::OnceCell, time::sleep};

static INSTANCE_ADDR: &str = "http://0.0.0.0:8079";

/// # SARETY
/// Only `kill_instance()` will do write operation, other will use protected
/// operation inside the `OnceCell`.
///
/// Only `server.await` and `monitor()` will run parallelly, but only one task can
/// return. `monitor()` use `initialized()` while `server.await` use
/// `get_or_init()`. These two operation is protected.
///
/// Only if `monitor()` is finished, `kill_instance()` can be call. `server.await`
/// can not continue if `monitor()` is finished. `kill_instance()` can access
/// and take the ownership, so that next time call `get_or_init` will init a
/// new OnceCell.
///
/// Only if `kill_instance()` is finished, next `select!` can run, i.e., next
/// `get_or_init()` can be called.
static mut INSTANCE: OnceCell<u32> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    let default_addr = "0.0.0.0:8080";
    let addr: SocketAddr = default_addr.parse().unwrap();

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr().ip();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(remote_addr, req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);
    tokio::pin!(server);
    loop {
        tokio::select! {
            _ = &mut server => {
                continue;
            }
            _ = monitor() => {
                kill_instance().await;
            }
        }
    }
}

async fn monitor() {
    // wait 10 minitues
    const IDLE_TIME: u64 = 60_0;

    unsafe {
        while !INSTANCE.initialized() {
            sleep(Duration::from_secs(IDLE_TIME)).await;
        }
        // find instance initialized
        // start to sleep until
        if INSTANCE.initialized() {
            sleep(Duration::from_secs(IDLE_TIME)).await;
        }
    }
}

async fn kill_instance() {
    unsafe {
        Command::new("kill")
            .arg("-9")
            .arg(INSTANCE.take().unwrap().to_string().as_str())
            .spawn()
            .unwrap()
            .wait()
            .await
            .unwrap();
    }
}

/// Send the request to the real instance
async fn handle(client_ip: IpAddr, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // check whether instance exsit
    // Safety: make sure only one instance can start and only after instance start,
    // request can be response
    unsafe {
        let _ = INSTANCE
            .get_or_init(|| async move {
                let start_time = SystemTime::now();
                println!(
                    "Init instance at: {}",
                    start_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
                );

                let child = tokio::spawn(async move {
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
                    if let Ok(_) = hyper_reverse_proxy::call(
                        client_ip,
                        INSTANCE_ADDR,
                        Request::new(Body::empty()),
                    )
                    .await
                    {
                        break;
                    }
                    if i == MAX_TRY {
                        panic!("Unable to boot instance function")
                    }
                }

                child.id().unwrap()
            })
            .await;
    }

    match hyper_reverse_proxy::call(client_ip, INSTANCE_ADDR, req).await {
        Ok(response) => Ok(response),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()),
    }
}
