use crypto_box::{Nonce as CryptoNonce, PublicKey, SecretKey};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct JsonNonce(CryptoNonce);

impl From<[u8; 24]> for JsonNonce {
    fn from(value: [u8; 24]) -> Self {
        Self(CryptoNonce::from(value))
    }
}

impl AsRef<[u8]> for JsonNonce {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
/// Peer Identity used for handshake
#[derive(Debug, Deserialize)]
pub struct Identity {
    pub peer_id: String,
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
    pub proof_of_work_stamp: JsonNonce,
}
