/// Ser/de for Tezos p2p messages
///
use serde::{Deserialize, Serialize};

use crate::p2p::{Nonce, PublicKey};

use crate::encoding::bin::BuffVisitor;

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_seq(BuffVisitor::<24>)?;
        Ok(Self::from(bytes))
    }
}
impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_seq(BuffVisitor::<32>)?;
        Ok(Self::from(bytes))
    }
}
impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        encoding::bin::to_bytes,
        p2p::{ChainName, ConnectionMessage},
    };

    #[test]
    fn it_serializes_connection_message() -> Result<()> {
        let conn_msg = ConnectionMessage {
            port: 9732,
            public_key: [0xf2; 32].into(),
            proof_of_work_stamp: [1; 24].into(),
            nonce: [2; 24].into(),
            chain_name: ChainName::default(),
            distributed_db_version: 2,
            p2p_version: 1,
        };
        let res = to_bytes(&conn_msg)?;
        assert_eq!(128, res.len());
        assert_eq!(&res[0..2], &[0, 126]);
        assert_eq!(
            &res[2..4],
            &[(conn_msg.port >> 8) as u8, (conn_msg.port & 0xff) as u8]
        );
        assert_eq!(&res[4..36], (&conn_msg.public_key).as_ref());
        assert_eq!(&res[36..60], (&conn_msg.proof_of_work_stamp).as_ref());
        assert_eq!(&res[60..84], (&conn_msg.nonce).as_ref());
        assert_eq!(&res[84..88], &[0, 0, 0, 36]);
        assert_eq!(&res[88..124], (&conn_msg.chain_name).as_ref());
        assert_eq!(&res[124..126], &[0, 2]);
        assert_eq!(&res[126..128], &[0, 1]);

        Ok(())
    }
}
