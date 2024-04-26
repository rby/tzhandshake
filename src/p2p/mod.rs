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
    /// Increments nonces by 1 as if they were u184 (8 * 24)
    /// ```rust
    /// use tzhandhsake::p2p::Nonce;
    /// let mut nonce = Nonce::from([0xff;24]);
    /// nonce.inc();
    /// assert_eq!(nonce.as_ref(), &[0;24]);
    /// ```
    pub fn inc(&mut self) {
        self.inc_step(1);
    }
    pub fn inc_step(&mut self, step: u16) {
        self.inc_byteno(22, step);
    }
    fn inc_byteno(&mut self, byteno: usize, step: u16) {
        assert!(byteno < 24, "overflow");
        assert!(byteno % 2 == 0, "byteno should be even");
        let mut step = step as u32;
        let mut byteno = byteno;
        loop {
            // equivalent to OCaml Bytes.get_uint15_be
            let mut res = ((self.0[byteno] as u32) << 8) + (self.0[byteno + 1] as u32);
            res += step;
            let lo = res & 0xffff;
            let hi = res >> 16;
            self.0[byteno] = (lo >> 8) as u8;
            self.0[byteno + 1] = lo as u8;
            if step == 0 || byteno == 0 {
                break;
            }
            byteno -= 2;
            step = hi;
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

/// Metadata is actuall 2 booleans
/// `src/lib_p2p_services/connection_metadata.ml`
/// (disable_mempool, private_node)
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

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use super::Nonce;

    /// I'm actually wondering if this is not actually the most performant
    /// implementation.
    /// ```text
    /// example::basic_inc::hc34e38a139aa478b:
    ///         test    sil, sil
    ///         je      .LBB0_4
    ///         mov     eax, 23
    /// .LBB0_2:
    ///         cmp     rax, -1
    ///         je      .LBB0_4
    ///         add     byte ptr [rdi + rax], sil
    ///         lea     rax, [rax - 1]
    ///         mov     sil, 1
    ///         jb      .LBB0_2
    /// .LBB0_4:
    ///         ret
    /// ```
    fn basic_inc<const N: usize>(nonce: &mut [u8; N], step: u8) {
        if step == 0 {
            return;
        };
        nonce
            .iter_mut()
            .rev()
            .try_fold(step, |acc, x| match x.overflowing_add(acc) {
                (res, true) => {
                    *x = res;
                    Some(1)
                }
                (res, false) => {
                    *x = res;
                    None
                }
            });
    }
    #[rustfmt::skip]
    fn to_u32(src: &[u8; 4]) -> u32 {
        (src[0] as u32) << 24 |
        (src[1] as u32) << 16 |
        (src[2] as u32) << 8  |
        (src[3] as u32)
    }

    proptest! {
        // We can verify the basic implementation is correct for a size of 2 or 4
        // and then once we're comfortable that the `basic_inc` is correct, we can use it
        // to proptest the real implementation
        #[test]
        fn it_propcheck_basic_inc_for_u16(nonce: [u8; 4], step: u8) {
            let value = to_u32(&nonce);
            let (expected, _) = value.overflowing_add(step as u32);
            let mut copy = nonce;
            basic_inc(&mut copy, step);
            let incremented = to_u32(&copy);
            assert_eq!(expected, incremented);
        }

        #[test]
        fn it_prop_check_inc_for_u184(arr: [u8;24], step: u8) {

            let mut copy =  arr ;

            let mut nonce = Nonce::from(arr);
            basic_inc(&mut copy, step);
            nonce.inc_step(step as u16);
            let res = nonce.as_ref();
            let (_, copy_u8, _) = unsafe {copy.align_to::<u8>()};
            assert_eq!(copy_u8, res);
        }
    }
}
