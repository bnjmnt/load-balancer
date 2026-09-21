use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;

const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(3);

pub struct Backend {
    pub addr: String,
    alive: AtomicBool,
    active_connections: AtomicUsize,
}

pub struct Balancer {
    backends: Vec<Backend>,
}

pub struct ConnectionGuard<'a> {
    backend: &'a Backend,
}

impl<'a> Drop for ConnectionGuard<'a> {
    fn drop(&mut self) {
        self.backend
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }
}

impl Balancer {
    pub fn new(addrs: &[&str]) -> Self {
        Balancer {
            backends: addrs
                .iter()
                .map(|s| Backend {
                    addr: s.to_string(),
                    alive: AtomicBool::new(true),
                    active_connections: AtomicUsize::new(0),
                })
                .collect(),
        }
    }

    pub fn pick(&self) -> Option<(String, ConnectionGuard<'_>)> {
        loop {
            let chosen = self
                .backends
                .iter()
                .filter(|b| b.alive.load(Ordering::Relaxed))
                .min_by_key(|b| b.active_connections.load(Ordering::Relaxed))?;

            let current = chosen.active_connections.load(Ordering::Relaxed);

            if chosen
                .active_connections
                .compare_exchange(current, current + 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return Some((chosen.addr.clone(), ConnectionGuard { backend: chosen }));
            }
        }
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
