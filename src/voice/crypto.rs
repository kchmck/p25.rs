use trunking::decode::*;
use voice::consts::CRYPTO_CONTROL_BYTES;

pub struct CryptoControlFields([u8; CRYPTO_CONTROL_BYTES]);

impl CryptoControlFields {
    pub fn new(buf: [u8; 12]) -> Self { CryptoControlFields(buf) }

    pub fn crypto_init(&self) -> &[u8] { &self.0[..9] }

    pub fn crypto_alg(&self) -> Option<CryptoAlgorithm> {
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
}

impl CryptoAlgorithm {
    pub fn from_bits(bits: u8) -> Option<CryptoAlgorithm> {
        use self::CryptoAlgorithm::*;

        match bits {
            0x00 => Some(Accordion),
            0x01 => Some(BatonEven),
            0x02 => Some(Firefly),
            0x03 => Some(Mayfly),
            0x04 => Some(Saville),
            0x41 => Some(BatonOdd),
            0x80 => Some(Unencrypted),
            0x81 => Some(DES),
            0x83 => Some(TripleDES),
            0x84 => Some(AES),
            _ => None,
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
        assert_eq!(c.crypto_alg(), Some(AES));
        assert_eq!(c.crypto_key(), 0xDEAD);
    }
}
