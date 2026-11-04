use aquamarine_core::packets::{Packet, write_to_stream};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next(); // eat past binary name

    let addr = args.next().unwrap_or("127.0.0.1:2946".to_string());

    let mut stream = TcpStream::connect(addr)
        .await
        .expect("Failed to connect to server");

    let _ = write_to_stream(
        &mut stream,
        Packet::Set {
            key: "test".to_string(),
            value: "456".to_string(),
        },
    )
    .await;
}
