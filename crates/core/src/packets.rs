use std::io::{Error, ErrorKind, Read, Write};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Serialize, Deserialize)]
pub enum Packet {
    Set { key: String, value: String },
    Get { key: String },
    Delete { key: String },
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Writes a length-prefixed payload to the stream.
pub fn write_to_stream<S, P>(stream: &mut S, payload: P) -> Result<()>
where
    S: Write,
    P: Serialize,
{
    let mut buffer = [0u8; 1024];

    // Serialize payload into buffer
    let serialized_bytes = postcard::to_slice(&payload, &mut buffer)
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    let len = serialized_bytes.len() as u32;

    // Send the length prefix (4 bytes, big-endian)
    stream.write_all(&len.to_be_bytes())?;

    // Send only the actual serialized bytes
    stream.write_all(serialized_bytes)?;

    Ok(())
}

/// Reads a length-prefixed payload from the stream.
pub fn read_from_stream<S, P>(stream: &mut S) -> Result<P>
where
    S: Read,
    P: DeserializeOwned,
{
    // Read length prefix
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Allocate buffer for payload (or use stack buffer if len <= MAX_SIZE)
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    // Deserialize payload
    let payload =
        postcard::from_bytes(&buffer).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    Ok(payload)
}
