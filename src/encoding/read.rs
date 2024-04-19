use crate::encoding::bin::from_bytes;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncReadExt;

#[derive(Debug, Error)]
enum ReadError {}

/// Trait that allows reading serialized `T` messages from an `AsyncReadExt.
/// Includes possibility of reading header-prefixed messages, and returning the buffer as well.
#[async_trait]
pub trait Read<T> {
    /// Reads from data that doesn't include the header, specify the size instead.
    /// This is typically for encrypted data, where the encrypted msg is the same size as the
    /// original message, hence its size can be deduced as the length of the TAG subtracted
    /// from the total length of the message
    async fn read<R>(r: &mut R, size: usize) -> Result<T>
    where
        R: AsyncReadExt + Unpin + Send;

    /// Read and deserialize `T` from the reader, return the bytes buffer as well including the
    /// 2 bytes header.
    async fn full_read_buffer<R>(r: &mut R) -> Result<(T, Vec<u8>)>
    where
        R: AsyncReadExt + Unpin + Send,
    {
        let size = r.read_u16().await?;
        let mut buffer = vec![0; 2 + size as usize];
        buffer[0] = (size >> 8) as u8;
        buffer[1] = (size & 0xff) as u8;
        r.read_exact(&mut buffer[2..]).await?;
        let t = Self::read(&mut &buffer[2..], size as usize).await?;
        Ok((t, buffer))
    }

    /// Read and deserialize `T` from `r`, don't return the buffer
    async fn full_read<R>(r: &mut R) -> Result<T>
    where
        R: AsyncReadExt + Unpin + Send,
    {
        let size = r.read_u16().await?;
        let t = Self::read(r, size as usize).await?;
        Ok(t)
    }
}

#[async_trait]
impl<T> Read<T> for T
where
    T: for<'de> Deserialize<'de>,
{
    async fn read<R>(r: &mut R, size: usize) -> Result<T>
    where
        R: AsyncReadExt + Unpin + Send,
    {
        let mut buffer = vec![0; size];
        r.read_exact(&mut buffer).await?;
        let t = from_bytes(&mut buffer)?;
        Ok(t)
    }
}
