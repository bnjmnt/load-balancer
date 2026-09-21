use crate::balancer::Balancer;
use tokio::io;
use tokio::net::TcpStream;

pub async fn handle_connection(mut client: TcpStream, balancer: &Balancer) -> io::Result<()> {
    let backend_addr = balancer
        .pick()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no healthy backends available"))?;
    let mut backend = TcpStream::connect(&backend_addr).await?;
    io::copy_bidirectional(&mut client, &mut backend).await?;
    Ok(())
}
