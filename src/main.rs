mod balancer;
mod proxy;

use balancer::Balancer;
use proxy::handle_connection;
use std::sync::Arc;
use tokio::io;
use tokio::net::TcpListener;

const LISTEN_ADDR: &str = "127.0.0.1:8080";
const BACKEND_ADDRS: &[&str] = &["127.0.0.1:9001", "127.0.0.1:9002", "127.0.0.1:9003"];

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    println!("listening on {LISTEN_ADDR}");

    let balancer = Arc::new(Balancer::new(BACKEND_ADDRS));

    tokio::spawn(Arc::clone(&balancer).run_health_checks());

    loop {
        let (client, addr) = listener.accept().await?;
        let balancer = Arc::clone(&balancer);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(client, &balancer).await {
                eprint!("connection from {addr} failed: {e}");
            }
        });
    }
}
