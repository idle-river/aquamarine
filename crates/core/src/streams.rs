use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_FRAME_LEN: u32 = 16 * 1024 * 1024;

pub async fn write_to_stream<S, P>(stream: &mut S, payload: &P) -> Result<()>
where
    S: AsyncWrite + Unpin,
    P: Serialize,
{
    let bytes = postcard::to_allocvec(payload)?;
    let len = u32::try_from(bytes.len())?;

    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

pub async fn read_from_stream<S, P>(stream: &mut S) -> Result<P>
where
    S: AsyncRead + Unpin,
    P: DeserializeOwned,
{
    let length = stream.read_u32_le().await?;
    if length > MAX_FRAME_LEN {
        return Err("frame too large: {length} bytes".into());
    }

    let mut bytes = vec![0u8; length as usize];
    stream.read_exact(&mut bytes).await?;

    Ok(postcard::from_bytes(&bytes)?)
}
