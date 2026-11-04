use std::io::{self, Write};

use aquamarine_core::packets::{
    ClientPacket, Packet, ServerPacket, read_from_stream, write_to_stream,
};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next(); // eat past binary name

    let addr = args.next().unwrap_or("127.0.0.1:2946".to_string());

    let mut stream = TcpStream::connect(&addr)
        .await
        .expect("Failed to connect to server");

    println!("Aquamarine CLI v1.0");
    println!("Connected to: marine://{}", addr);

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();

        let cmd = cmd.trim();
        let mut cmd = cmd.split_whitespace();

        let payload = match cmd.next().unwrap().to_uppercase().as_ref() {
            "EXIT" => break,
            "GET" => {
                let key = cmd.next().unwrap();

                ClientPacket::Get {
                    key: key.to_string(),
                }
            }
            "DEL" | "DELETE" => {
                let key = cmd.next().unwrap();

                ClientPacket::Delete {
                    key: key.to_string(),
                }
            }
            "SET" => {
                let key = cmd.next().unwrap();
                let value = cmd.next().unwrap();

                ClientPacket::Set {
                    key: key.to_string(),
                    value: value.to_string(),
                }
            }
            _ => unreachable!(),
        };

        if let Err(e) =
            write_to_stream::<Packet, TcpStream>(&mut stream, Packet::Client(payload)).await
        {
            println!("Failed to send payload: {e}");
            continue;
        };

        match read_from_stream::<Packet, TcpStream>(&mut stream).await {
            Ok(p) => match p {
                Packet::Ping => {
                    let _ = write_to_stream::<Packet, TcpStream>(&mut stream, Packet::Pong).await;
                }
                Packet::Pong => {}

                Packet::Server(s) => match s {
                    ServerPacket::GetRes { value } => println!("{value}"),
                    ServerPacket::SetRes | ServerPacket::DeleteRes => {}
                },

                // Client packets
                Packet::Client(_) => unreachable!(),
            },
            Err(e) => {
                println!("Failed to read response: {e}");
            }
        };
    }
}
