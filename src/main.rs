use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

const LISTEN_ADDR: &str = "127.0.0.1:8080";
const BACKEND_ADDRS: &[&str] = &["127.0.0.1:9001", "127.0.0.1:9002", "127.0.0.1:9003"];

struct Balancer {
    backends: Vec<String>,
    next: AtomicUsize,
}

impl Balancer {
    fn new(backends: &[&str]) -> Self {
        Balancer {
            backends: backends.iter().map(|s| s.to_string()).collect(),
            next: AtomicUsize::new(0),
        }
    }

    fn pick(&self) -> String {
        let i = self.next.fetch_add(1, Ordering::Relaxed) % self.backends.len();
        self.backends[i].clone()
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    println!("listening on {LISTEN_ADDR}");

    let balancer = Arc::new(Balancer::new(BACKEND_ADDRS));

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

async fn handle_connection(mut client: TcpStream, balancer: &Balancer) -> io::Result<()> {
    let backend_addr = balancer.pick();
    let mut backend = TcpStream::connect(backend_addr).await?;
    io::copy_bidirectional(&mut client, &mut backend).await?;
    Ok(())
}
