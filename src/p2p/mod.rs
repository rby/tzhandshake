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

impl Nonce {
    /// Increments nonces as u16s
    /// Copied from the ML implementation
    pub fn inc(&mut self) {
        self.inc_step(1);
    }
    pub fn inc_step(&mut self, step: u16) {
        self.inc_byteno(22, step);
    }
    fn inc_byteno(&mut self, byteno: usize, step: u16) {
        assert!(byteno < 24, "overflow");
        assert!(byteno % 2 == 0, "byteno should be even");

        // equivalent to OCaml Bytes.get_uint15_be
        let mut res = ((self.0[byteno] as u32) << 8) + (self.0[byteno + 1] as u32);
        res += step as u32;
        let lo = res & 0xffff;
        let hi = res >> 16;
        self.0[byteno] = (lo >> 8) as u8;
        self.0[byteno + 1] = (lo & 0xff) as u8;
        if hi != 0 && byteno != 0 {
            self.inc_byteno(byteno - 2, hi as u16)
        }
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

#[derive(Debug, PartialEq, Default)]
pub struct Metadata([u8; 2]);

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

impl ConnectionMessage {
    fn public_key(&self) -> &crypto_box::PublicKey {
        &self.public_key.0
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Ack(bool);
