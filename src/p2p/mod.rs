use crypto_box;
use serde::{Deserialize, Serialize};

pub mod binserde;

/// Newtype for Nonce, allowing implementation of binary serialization
/// when transferred in p2p messages
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Nonce(crypto_box::Nonce);

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnectionMessage {
    pub(crate) port: u16,
    pub(crate) public_key: PublicKey,
    pub(crate) proof_of_work_stamp: Nonce,
    pub(crate) nonce: Nonce,
    pub(crate) chain_name: ChainName,
    pub(crate) distributed_db_version: u16,
    pub(crate) p2p_version: u16,
}
