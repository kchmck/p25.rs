use consts::CRYPTO_CONTROL_BYTES;
use util::slice_u16;

pub struct CryptoControlFields([u8; CRYPTO_CONTROL_BYTES]);

impl CryptoControlFields {
    pub fn new(buf: [u8; 12]) -> Self { CryptoControlFields(buf) }

    pub fn crypto_init(&self) -> &[u8] { &self.0[..9] }

    pub fn crypto_alg(&self) -> CryptoAlgorithm {
        CryptoAlgorithm::from_bits(self.0[9])
    }

    pub fn crypto_key(&self) -> u16 { slice_u16(&self.0[10..]) }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CryptoAlgorithm {
    Accordion,
    BatonEven,
    Firefly,
    Mayfly,
    Saville,
    BatonOdd,
    Unencrypted,
    DES,
    TripleDES,
    AES,
    Other(u8),
}

impl CryptoAlgorithm {
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
            0x81 => DES,
            0x83 => TripleDES,
            0x84 => AES,
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

        assert_eq!(c.crypto_init(), &[0,0,0,1,0,0,0,2,0]);
        assert_eq!(c.crypto_alg(), AES);
        assert_eq!(c.crypto_key(), 0xDEAD);
    }
}
