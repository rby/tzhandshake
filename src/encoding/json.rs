/// Json serialization for configurations and identities
use serde::{
    de::{self},
    ser, Deserialize, Serialize,
};

use crate::identity::JsonNonce;

impl<'de> Deserialize<'de> for JsonNonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = [0u8; 24];
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Ok(Self::from(bytes))
    }
}

impl Serialize for JsonNonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(&self, serializer)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use crate::identity::Identity;

    #[test]
    fn it_deserializes_peer_id_json() {
        let data = json!({
            "peer_id": "idrpbo9Ru5pYiWTg1i2VPABG6Catfm",
            "public_key": "3b2c3950d9c59a5c19af7be39ce5844523bc002651cd45417e635462ce666f07",
            "secret_key": "d0c24b1537d8651ebc39951030c1fccea83188d563ac8cf9667f7ccb7765b1ba",
            "proof_of_work_stamp": "ea2fa50b542755be6bc4a53188d758cf4e7d4e085082f4bd"
        });
        let peer_id = Identity::deserialize(data).expect("should deserialize");
        assert_eq!(
            "idrpbo9Ru5pYiWTg1i2VPABG6Catfm".to_string(),
            peer_id.peer_id
        )
    }
}
