use crate::encoding::bin::from_bytes;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::AsyncReadExt;

#[derive(Debug, Error)]
enum ReadError {}

/// Trait that allows reading serialized `T` messages from an `AsyncReadExt`.
/// Includes possibility of reading header-prefixed messages, and returning the buffer as well.
#[async_trait]
pub trait Read<T> {
    /// Reads non heade-prefixed data, specify the size instead.
    /// This is used for encrypted data, where the full encrypted msg has the following format:
    /// | header | tag | encrypted |
    /// where header = tag.len() (16) + encrypted.len()
    async fn read<R>(r: &mut R, size: usize) -> Result<T>
    where
        R: AsyncReadExt + Unpin + Send;

    /// Read and deserialize `T` from `r` that contains the header-prefixed data.
    /// Return also all the bytes that were read.
    async fn full_read_buffer<R>(r: &mut R) -> Result<(T, Vec<u8>)>
    where
        R: AsyncReadExt + Unpin + Send,
    {
        let size = r.read_u16().await?;
        let mut buffer = vec![0; 2 + size as usize];
        buffer[0] = (size >> 8) as u8;
        buffer[1] = size as u8;
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
