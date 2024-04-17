/// Ser/de for Tezos p2p messages
///
use serde::{Deserialize, Serialize};

use crate::p2p::{Nonce, PublicKey};

use super::bin::BuffVisitor;

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = deserializer.deserialize_bytes(BuffVisitor::<24>)?;
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
        let bytes = deserializer.deserialize_bytes(BuffVisitor::<32>)?;
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
        assert_eq!(134, res.len());
        assert_eq!(&res[0..2], &[0, 132]);
        assert_eq!(
            &res[2..4],
            &[(conn_msg.port >> 8) as u8, (conn_msg.port & 0xff) as u8]
        );
        assert_eq!(&res[4..6], &[0, 32]);
        assert_eq!(&res[6..38], (&conn_msg.public_key).as_ref());
        assert_eq!(&res[38..40], &[0, 24]);
        assert_eq!(&res[40..64], (&conn_msg.proof_of_work_stamp).as_ref());
        assert_eq!(&res[64..66], &[0, 24]);
        assert_eq!(&res[66..90], (&conn_msg.nonce).as_ref());
        assert_eq!(&res[90..94], &[0, 0, 0, 36]);
        assert_eq!(&res[94..130], (&conn_msg.chain_name).as_ref());
        assert_eq!(&res[130..132], &[0, 2]);
        assert_eq!(&res[132..134], &[0, 1]);

        Ok(())
    }
}
