use crypto_box::{Nonce as CryptoNonce, PublicKey, SecretKey};
use serde::{de, Deserialize};

#[derive(Debug)]
struct Nonce(CryptoNonce);

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = [0u8; 24];
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Ok(Nonce(CryptoNonce::from(bytes)))
    }
}

/// Peer Identity used for handshake
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct PeerId {
    peer_id: String,
    public_key: PublicKey,
    secret_key: SecretKey,
    proof_of_work_stamp: Nonce,
}

#[cfg(test)]
mod test {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[test]
    fn it_deserializes_peer_id() {
        let content = json!({
            "peer_id" : "idrpbo9Ru5pYiWTg1i2VPABG6Catfm",
            "public_key": "0000000000000000000000000000000000000000000000000000000000000000",
            "secret_key": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "proof_of_work_stamp": "eaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaeaea",
        });
        let res = PeerId::deserialize(content);
        assert!(res.is_ok())
    }
}
