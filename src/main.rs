mod balancer;
mod proxy;

use balancer::Balancer;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use proxy::{ProxyClient, handle_request};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

const LISTEN_ADDR: &str = "127.0.0.1:8080";
const BACKEND_ADDRS: &[&str] = &["127.0.0.1:9001", "127.0.0.1:9002", "127.0.0.1:9003"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = LISTEN_ADDR.parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {LISTEN_ADDR}");

    let balancer = Arc::new(Balancer::new(BACKEND_ADDRS));
    tokio::spawn(Arc::clone(&balancer).run_health_checks());

    let client: ProxyClient = Client::builder(TokioExecutor::new()).build_http();

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let balancer = Arc::clone(&balancer);
        let client = client.clone();

        tokio::spawn(async move {
            let service =
                service_fn(move |req| handle_request(req, Arc::clone(&balancer), client.clone()));
            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprint!("error serving connection: {:?}", err);
            }
        });
    }
}
