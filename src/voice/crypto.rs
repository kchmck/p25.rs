//! Decode Cryptographic Control (CC) packets.

use consts::CRYPTO_CONTROL_BYTES;
use util::slice_u16;

/// Buffer of bytes that represent a crypto control packet.
pub type Buf = [u8; CRYPTO_CONTROL_BYTES];

/// Information necessary to decrypt an encrypted message.
pub struct CryptoControlFields(Buf);

impl CryptoControlFields {
    /// Create a new `CryptoControlFields` decoder from the given bytes.
    pub fn new(buf: Buf) -> Self { CryptoControlFields(buf) }

    /// Initialization vector used internally by associated crypto algorithm.
    pub fn init(&self) -> &[u8] { &self.0[..9] }
    /// Type of crypto algorithm in use, if any.
    pub fn alg(&self) -> CryptoAlgorithm { CryptoAlgorithm::from_bits(self.0[9]) }
    /// Encryption key to use.
    pub fn key(&self) -> u16 { slice_u16(&self.0[10..]) }
}

/// Type of cryptographic algorithm.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "ser", derive(Serialize))]
pub enum CryptoAlgorithm {
    Accordion,
    BatonEven,
    Firefly,
    Mayfly,
    Saville,
    BatonOdd,
    Unencrypted,
    Des,
    TripleDes,
    Aes,
    Other(u8),
}

impl CryptoAlgorithm {
    /// Parse the given 8 bits into a crypto algorithm.
    pub fn from_bits(bits: u8) -> CryptoAlgorithm {
        use self::CryptoAlgorithm::*;

        match bits {
            0x00 => Accordion,
            0x01 => BatonEven,
            0x02 => Firefly,
            0x03 => Mayfly,
            0x04 => Saville,
            0x41 => BatonOdd,
            0x80 => Unencrypted,
            0x81 => Des,
            0x83 => TripleDes,
            0x84 => Aes,
            b => Other(b),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::CryptoAlgorithm::*;

    #[test]
    fn test_cypto() {
        let c = CryptoControlFields::new([
            0, 0, 0, 1, 0, 0, 0, 2, 0,
            0b10000100,
            0xDE, 0xAD,
        ]);

        assert_eq!(c.init(), &[0,0,0,1,0,0,0,2,0]);
        assert_eq!(c.alg(), Aes);
        assert_eq!(c.key(), 0xDEAD);
    }
}
