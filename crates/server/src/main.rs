use aquamarine_core::{
    KVStore,
    packets::{ClientPacket, Packet, ServerPacket, read_from_stream, write_to_stream},
};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:2946")
        .await
        .expect("Failed to bind to port 2946");

    println!("Server listening on marine://0.0.0.0:2946");

    let store = Arc::new(RwLock::new(KVStore::new("./aquamarine.wal")));

    while let Ok((stream, addr)) = listener.accept().await {
        let store = Arc::clone(&store);
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, addr, store).await {
                eprintln!("Error handling client: {e}");
            }
        });
    }
}

async fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
    store: Arc<RwLock<KVStore>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let payload = read_from_stream::<Packet, TcpStream>(&mut stream).await?;
        println!("Received packet from {addr}: {payload:?}");

        let response = match payload {
            Packet::Client(ClientPacket::Set { key, value }) => {
                let mut store = store
                    .write()
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                store.set(key, value)?;
                Packet::Server(ServerPacket::SetRes)
            }
            Packet::Client(ClientPacket::Get { key }) => {
                let value = store
                    .read()
                    .map_err(|error| std::io::Error::other(error.to_string()))?
                    .get(key)
                    .unwrap_or_default();
                Packet::Server(ServerPacket::GetRes { value })
            }
            Packet::Client(ClientPacket::Delete { key }) => {
                let mut store = store
                    .write()
                    .map_err(|error| std::io::Error::other(error.to_string()))?;
                store.delete(key)?;
                Packet::Server(ServerPacket::DeleteRes)
            }
            Packet::Ping => Packet::Pong,
            Packet::Pong => continue,
            Packet::Server(_) => unreachable!(),
        };

        write_to_stream(&mut stream, response).await?;
    }
}
