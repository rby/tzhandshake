use crypto_box::{self, aead::rand_core::CryptoRngCore};
use serde::{Deserialize, Serialize};

pub mod binserde;
pub mod handshake;

/// Newtype for Nonce, allowing implementation of binary serialization
/// when transferred in p2p messages
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Nonce(crypto_box::Nonce);

impl Nonce {
    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: CryptoRngCore,
    {
        let mut bytes = [0; 24];
        rng.fill_bytes(&mut bytes);
        Nonce::from(bytes)
    }
}

impl From<[u8; 24]> for Nonce {
    fn from(value: [u8; 24]) -> Self {
        Self(crypto_box::Nonce::from(value))
    }
}
impl AsRef<[u8]> for Nonce {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
impl Default for Nonce {
    fn default() -> Self {
        Nonce::from([0; 24])
    }
}

/// New type for bin serialization of PK
#[derive(Debug, PartialEq)]
pub struct PublicKey(crypto_box::PublicKey);

impl From<[u8; 32]> for PublicKey {
    fn from(value: [u8; 32]) -> Self {
        Self(crypto_box::PublicKey::from(value))
    }
}
impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl PublicKey {
    pub fn new(pk: crypto_box::PublicKey) -> Self {
        PublicKey(pk)
    }
}

impl Default for PublicKey {
    fn default() -> Self {
        PublicKey::from([0; 32])
    }
}

/// Ghostnet default chain name
const DEFAULT_CHAIN: &'static str = "TEZOS_ITHACANET_2022-01-25T15:00:00Z";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainName(String);
impl Default for ChainName {
    fn default() -> Self {
        Self(DEFAULT_CHAIN.to_string())
    }
}
impl AsRef<[u8]> for ChainName {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct P2PVersion(u16);

impl Default for P2PVersion {
    fn default() -> Self {
        P2PVersion(1)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DDBVersion(u16);

impl Default for DDBVersion {
    fn default() -> Self {
        DDBVersion(2)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ConnectionMessage {
    pub(crate) port: u16,
    pub(crate) public_key: PublicKey,
    pub(crate) proof_of_work_stamp: Nonce,
    pub(crate) nonce: Nonce,
    pub(crate) chain_name: ChainName,
    pub(crate) distributed_db_version: DDBVersion,
    pub(crate) p2p_version: P2PVersion,
}
