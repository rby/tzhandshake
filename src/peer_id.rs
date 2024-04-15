use crypto_box::{Nonce as CryptoNonce, PublicKey, SecretKey};
use serde::{de, ser, Deserialize, Serialize};

#[derive(Debug)]
pub struct Nonce(CryptoNonce);

impl From<[u8; 24]> for Nonce {
    fn from(value: [u8; 24]) -> Self {
        Nonce(CryptoNonce::from(value))
    }
}

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

impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(&self.0, serializer)
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
    #[test]
    fn it_serializes_nonces() -> Result<(), serde_json::Error> {
        let nonce = Nonce::from([5; 24]);
        let s = serde_json::to_string(&nonce)?;
        assert_eq!(s, "\"050505050505050505050505050505050505050505050505\"");
        Ok(())
    }
}
