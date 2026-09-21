use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(3);

pub struct Backend {
    pub addr: String,
    alive: AtomicBool,
}

pub struct Balancer {
    backends: Vec<Backend>,
    next: AtomicUsize,
}

impl Balancer {
    pub fn new(addrs: &[&str]) -> Self {
        Balancer {
            backends: addrs
                .iter()
                .map(|s| Backend {
                    addr: s.to_string(),
                    alive: AtomicBool::new(true),
                })
                .collect(),
            next: AtomicUsize::new(0),
        }
    }

    pub fn pick(&self) -> Option<String> {
        let len = self.backends.len();
        for _ in 0..len {
            let i = self.next.fetch_add(1, Ordering::Relaxed) % len;
            let backend = &self.backends[i];
            if backend.alive.load(Ordering::Relaxed) {
                return Some(backend.addr.clone());
            }
        }
        None
    }

    pub async fn run_health_checks(self: Arc<Self>) {
        let mut interval = time::interval(HEALTH_CHECK_INTERVAL);
        loop {
            interval.tick().await;
            for backend in &self.backends {
                let is_alive = TcpStream::connect(&backend.addr).await.is_ok();
                let was_alive = backend.alive.swap(is_alive, Ordering::Relaxed);
                if was_alive != is_alive {
                    println!(
                        "backend {} is now {}",
                        backend.addr,
                        if is_alive { "UP" } else { "DOWN" }
                    )
                }
            }
        }
    }
}
