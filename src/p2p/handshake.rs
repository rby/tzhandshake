/// Handshake module
///
use crate::{
    encoding::bin::to_bytes,
    identity::Identity,
    p2p::{ConnectionMessage, PublicKey},
};

use anyhow::Result;
use crypto_box::aead::rand_core::CryptoRngCore;
use thiserror::Error;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpStream, ToSocketAddrs},
};

use crate::encoding::read::Read;

use super::Nonce;

#[derive(Debug, Error)]
pub enum HandhshakeError {
    #[error("Missing Nonce")]
    MissingNonce,
}

#[derive(Debug)]
pub struct Handshake {
    identity: Identity,
    nonce: Option<Nonce>,
}

/// Kind of Builder pattern
impl Handshake {
    pub fn identity(identity: Identity) -> Self {
        Self {
            identity,
            nonce: None,
        }
    }
    pub fn generate_nonce<R>(mut self, rng: &mut R) -> Self
    where
        R: CryptoRngCore,
    {
        self.nonce = Some(Nonce::generate(rng));
        self
    }
    pub fn with_nonce(mut self, nonce: Nonce) -> Self {
        self.nonce = Some(nonce);
        self
    }
    pub async fn connect<A>(self, peer: A) -> Result<Channel>
    where
        A: ToSocketAddrs,
    {
        let mut stream = TcpStream::connect(peer).await?;
        let nonce = self.nonce.ok_or_else(|| HandhshakeError::MissingNonce)?;

        let sent_msg = ConnectionMessage {
            public_key: PublicKey::new(self.identity.public_key.clone()),
            nonce,
            proof_of_work_stamp: Nonce::from(self.identity.proof_of_work_stamp.bytes()),
            ..Default::default()
        };
        let sent_bytes = to_bytes(&sent_msg)?;

        let mut recv_msg_bytes = vec![];
        let _rcv_msg = ConnectionMessage::read(&mut stream, &mut recv_msg_bytes).await?;
        stream.write(&sent_bytes).await?;
        stream.flush().await?;
        Ok(Channel {
            identity: self.identity,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Channel {
    identity: Identity,
}
