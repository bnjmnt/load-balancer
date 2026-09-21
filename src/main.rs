use tokio::io;
use tokio::net::{TcpListener, TcpStream};

const LISTEN_ADDR: &str = "127.0.0.1:8080";
const BACKEND_ADDR: &str = "127.0.0.1:9001";

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    println!("listening on {LISTEN_ADDR}");

    loop {
        let (client, addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(client).await {
                eprint!("connection from {addr} failed: {e}");
            }
        });
    }
}

async fn handle_connection(mut client: TcpStream) -> io::Result<()> {
    let mut backend = TcpStream::connect(BACKEND_ADDR).await?;
    io::copy_bidirectional(&mut client, &mut backend).await?;
    Ok(())
}
