use std::{fs::File, io::BufReader, path::Path};

use anyhow::Result;
use crypto_box::{Nonce as CryptoNonce, PublicKey, SecretKey};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct JsonNonce(CryptoNonce);

impl From<[u8; 24]> for JsonNonce {
    fn from(value: [u8; 24]) -> Self {
        Self(CryptoNonce::from(value))
    }
}

impl JsonNonce {
    pub fn bytes(&self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes.copy_from_slice(&self.0[0..24]);
        bytes
    }
}

impl AsRef<[u8]> for JsonNonce {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
/// Peer Identity used for handshake
#[derive(Debug, Deserialize, Clone)]
pub struct Identity {
    pub peer_id: String,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub proof_of_work_stamp: JsonNonce,
}

impl Identity {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        let res = serde_json::from_reader(reader)?;
        Ok(res)
    }
}
