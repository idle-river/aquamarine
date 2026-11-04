use aquamarine_core::packets::{Packet, read_from_stream};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:2946")
        .await
        .expect("Failed to bind to port 2946");

    println!("Server listening on marine://0.0.0.0:2946");

    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr).await {
                eprintln!("Error handling client: {e}");
            }
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = read_from_stream::<Packet, TcpStream>(&mut stream).await?;
    println!("Received packet from {addr}: {payload:?}");
    Ok(())
}
