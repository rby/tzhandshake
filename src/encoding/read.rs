use crate::encoding::bin::from_bytes;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Error)]
enum ReadError {}

#[async_trait]
pub trait Read<'a, T, R> {
    async fn read(r: &'a mut R, buff: &'a mut Vec<u8>) -> Result<T>;
}

#[async_trait]
impl<'a, T, R> Read<'a, T, R> for T
where
    R: AsyncReadExt + Unpin + Send,
    T: Deserialize<'a>,
{
    async fn read(r: &'a mut R, buff: &'a mut Vec<u8>) -> Result<T> {
        let size = r.read_u16().await?;
        let mut inner = vec![0; size as usize];
        AsyncWriteExt::write_u16(buff, size).await?;
        r.read_exact(&mut inner).await?;
        AsyncWriteExt::write(buff, &inner).await?;
        let t = from_bytes(&mut buff[2..])?;
        Ok(t)
    }
}
