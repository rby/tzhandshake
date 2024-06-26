use std::marker::PhantomData;

/// Handshake module
///
use crate::{
    encoding::{
        self,
        bin::{to_bytes, to_bytes_no_header},
    },
    identity::Identity,
    p2p::{Ack, ConnectionMessage, PublicKey},
};

use anyhow::Result;
use async_trait::async_trait;
use blake2::digest::{consts::U32, Digest};
use blake2::Blake2b;
use crypto_box::aead::AeadMutInPlace;
use crypto_box::{aead::rand_core::CryptoRngCore, SalsaBox};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

use crate::encoding::read::Read;

use super::{Metadata, Nonce};

#[derive(Debug, Error)]
pub enum HandhshakeError {
    #[error("Missing Nonce")]
    MissingNonce,
    #[error("The encrypted message must at least be longer than a tag")]
    EncryptedMessageShorterThanTag,
}

#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Handhsake Error `{0}`")]
    Handshake(#[from] HandhshakeError),
    #[error("IO error (Tokio) `{0}`")]
    Network(#[from] tokio::io::Error),
    #[error("Crypto error `{0}`")]
    Crypto(String),
    #[error("Ser/Deserialization error `{0}`")]
    Serde(#[from] encoding::error::Error),
    #[error("Anyhow: `{0}`")]
    Anyhow(#[from] anyhow::Error),
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
    pub async fn connect<A>(self, peer: A) -> Result<Channel<TcpStream>>
    where
        A: ToSocketAddrs,
    {
        let mut stream = TcpStream::connect(peer).await?;
        let nonce = self.nonce.ok_or(HandhshakeError::MissingNonce)?;

        let sent = ConnectionMessage {
            public_key: PublicKey::new(self.identity.public_key.clone()),
            nonce,
            proof_of_work_stamp: Nonce::from(self.identity.proof_of_work_stamp.bytes()),
            ..Default::default()
        };
        let sent_bytes = to_bytes(&sent)?;

        let (received, received_bytes) = ConnectionMessage::full_read_buffer(&mut stream).await?;

        let received = ReceivedMsg::new(received, received_bytes);

        stream.write_all(&sent_bytes).await?;
        stream.flush().await?;
        let sent = SentMsg::new(sent, sent_bytes.to_vec());
        // this form of builder pattern is not great to be honest.
        let mut chan = Channel::new(
            stream,
            &self.identity,
            received,
            sent,
            ConnectionDirection::Outgoing,
        );
        // these are messages that seem to be exchanged to verify that we can encrypt/decrypt
        // correctly. Not sure why we don't that with Acks.
        let _metadata = chan.read_metadata().await?;
        println!("received metadata: {:?}", _metadata);
        chan.write_metadata().await?;

        let ack = chan.read::<Ack>().await?;
        println!("received ack: {:?}", ack);
        chan.write(Ack(true)).await?;

        Ok(chan)
    }
}

pub struct Channel<S> {
    stream: S,
    channel_key: SalsaBox,
    local_nonce: Nonce,
    remote_nonce: Nonce,
}

impl<S> Channel<S> {
    fn new(
        stream: S,
        identity: &Identity,
        received: Msg<ConnectionMessage, Received>,
        sent: Msg<ConnectionMessage, Sent>,
        direction: ConnectionDirection,
    ) -> Self {
        let channel_key =
            crypto_box::SalsaBox::new(received.value.public_key(), &identity.secret_key);

        // reorder the bytes to be fully deterministic before nonce computation
        let (init_bytes, resp_bytes) = match direction {
            ConnectionDirection::Incoming => (received.bytes, sent.bytes),
            ConnectionDirection::Outgoing => (sent.bytes, received.bytes),
        };
        let init_resp_nonce = compute_nonce(&init_bytes, &resp_bytes, b"Init -> Resp");
        let resp_init_nonce = compute_nonce(&init_bytes, &resp_bytes, b"Resp -> Init");
        let (local_nonce, remote_nonce) = match direction {
            ConnectionDirection::Incoming => (init_resp_nonce, resp_init_nonce),
            ConnectionDirection::Outgoing => (resp_init_nonce, init_resp_nonce),
        };

        Channel {
            stream,
            channel_key,
            local_nonce,
            remote_nonce,
        }
    }
}

impl<S> Channel<S>
where
    S: AsyncWriteExt + Send + Unpin,
{
    async fn write_metadata(&mut self) -> Result<(), P2PError>
    where
        S: AsyncWriteExt + Unpin,
    {
        self.write(&Metadata::default()).await
    }
}
impl<S> Channel<S>
where
    S: AsyncReadExt + Send + Unpin,
{
    async fn read_metadata(&mut self) -> Result<Metadata, P2PError>
    where
        S: AsyncReadExt + Unpin,
    {
        self.read::<Metadata>().await
    }
}
const TAG_LENGTH: u16 = 16;

#[async_trait]
pub trait TezosRead {
    async fn read<T>(&mut self) -> Result<T, P2PError>
    where
        T: Send + for<'de> Deserialize<'de>;
}

#[async_trait]
pub trait TezosWrite {
    async fn write<T>(&mut self, value: T) -> Result<(), P2PError>
    where
        T: Send + Serialize;
}

impl<S> Channel<S> {
    fn inc_local(&mut self) {
        self.local_nonce.inc()
    }
    fn inc_remote(&mut self) {
        self.remote_nonce.inc()
    }
}

#[async_trait]
impl<S> TezosRead for Channel<S>
where
    S: AsyncReadExt + Unpin + Send,
{
    async fn read<T>(&mut self) -> Result<T, P2PError>
    where
        T: Send + for<'de> Deserialize<'de>,
    {
        let mut header = self.stream.read_u16().await?;
        if header < TAG_LENGTH {
            return Err(HandhshakeError::EncryptedMessageShorterThanTag.into());
        }
        let mut tag = [0; TAG_LENGTH as usize];
        self.stream.read_exact(&mut tag).await?;
        header -= TAG_LENGTH;

        let mut encrypted = vec![0; header as usize];
        self.stream.read_exact(&mut encrypted).await?;
        self.channel_key
            .decrypt_in_place_detached(&self.remote_nonce.0, &[0; 0], &mut encrypted, &tag.into())
            .map_err(|s| P2PError::Crypto(s.to_string()))?;
        let recv = T::read(&mut encrypted.as_ref(), header as usize).await?;
        self.inc_remote();
        Ok(recv)
    }
}
#[async_trait]
impl<S> TezosWrite for Channel<S>
where
    S: AsyncWriteExt + Unpin + Send,
{
    async fn write<T>(&mut self, value: T) -> Result<(), P2PError>
    where
        T: Send + Serialize,
    {
        let mut buffer = to_bytes_no_header(&value)?;
        let tag = self
            .channel_key
            .encrypt_in_place_detached(&self.local_nonce.0, &[0; 0], &mut buffer)
            .map_err(|s| P2PError::Crypto(s.to_string()))?;
        let size = tag.len() + buffer.len();
        // is this  a programming error?
        assert!(
            size <= u16::max_value() as usize,
            "breaking protocol with msg too big",
        );
        self.stream.write_u16(size as u16).await?;
        self.stream.write_all(&tag).await?;
        self.stream.write_all(&buffer).await?;
        self.stream.flush().await?;
        // what happened if we fail before this incremnt.
        // The best for the node is to close the channel, and redo a handshake
        self.inc_local();
        Ok(())
    }
}

fn compute_nonce(sent: &[u8], recv: &[u8], seed: &[u8]) -> Nonce {
    type Blake2b256 = Blake2b<U32>;
    let res = Blake2b256::digest([sent, recv, seed].concat());
    let mut bytes = [0; 24];
    bytes.copy_from_slice(&res[..24]);
    Nonce::from(bytes)
}

enum ConnectionDirection {
    #[allow(dead_code)]
    Incoming,
    Outgoing,
}
struct Msg<A, T> {
    value: A,
    bytes: Vec<u8>,
    phantom: PhantomData<T>,
}
impl<A, T> Msg<A, T> {
    fn new(value: A, bytes: Vec<u8>) -> Self {
        Msg {
            value,
            bytes,
            phantom: PhantomData,
        }
    }
}

type SentMsg<A> = Msg<A, Sent>;
type ReceivedMsg<A> = Msg<A, Received>;

struct Sent;
struct Received;
