use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientPacket {
    Set { key: String, value: String },
    Get { key: String },
    Delete { key: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerPacket {
    GetRes { value: String },
    SetRes,
    DeleteRes,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Packet {
    Client(ClientPacket),
    Server(ServerPacket),
    Ping,
    Pong,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn write_to_stream<P, S>(stream: &mut S, payload: P) -> Result<()>
where
    P: Serialize,
    S: AsyncWrite + Unpin,
{
    let mut buffer = [0u8; 1024];

    // Serialize payload into buffer
    let serialized_bytes = postcard::to_slice(&payload, &mut buffer)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    let len = serialized_bytes.len() as u32;

    // Send the length prefix (4 bytes, big-endian)
    stream.write_all(&len.to_be_bytes()).await?;

    // Send only the actual serialized bytes
    stream.write_all(serialized_bytes).await?;

    Ok(())
}

pub async fn read_from_stream<P, S>(stream: &mut S) -> Result<P>
where
    P: DeserializeOwned,
    S: AsyncRead + Unpin,
{
    // Read length prefix
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Allocate buffer for payload (or use stack buffer if len <= MAX_SIZE)
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await?;

    // Deserialize payload
    let payload =
        postcard::from_bytes(&buffer).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    Ok(payload)
}
