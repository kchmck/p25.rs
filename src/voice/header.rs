use trunking::decode::*;
use voice::crypto;
use consts::HEADER_BYTES;

pub struct VoiceHeaderFields([u8; HEADER_BYTES]);

impl VoiceHeaderFields {
    pub fn new(buf: [u8; HEADER_BYTES]) -> Self { VoiceHeaderFields(buf) }

    pub fn crypto_init(&self) -> &[u8] { &self.0[..9] }
    pub fn mfg(&self) -> u8 { self.0[9] }

    pub fn crypto_alg(&self) -> crypto::CryptoAlgorithm {
        crypto::CryptoAlgorithm::from_bits(self.0[10])
    }

    pub fn crypto_key(&self) -> u16 { slice_u16(&self.0[11..]) }

    pub fn talk_group(&self) -> TalkGroup {
        TalkGroup::from_bits(slice_u16(&self.0[13..]))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use voice::crypto::CryptoAlgorithm::*;
    use trunking::decode::TalkGroup;

    #[test]
    fn test_header() {
        let h = VoiceHeaderFields::new([
            1, 2, 3, 4, 5, 6, 7, 8, 9,
            0b00000000,
            0b10000000,
            0b00000000,
            0b00000000,
            0b11111111,
            0b11111111,
        ]);

        assert_eq!(h.crypto_init(), &[1,2,3,4,5,6,7,8,9]);
        assert_eq!(h.mfg(), 0);
        assert_eq!(h.crypto_alg(), Unencrypted);
        assert_eq!(h.crypto_key(), 0);
        assert_eq!(h.talk_group(), TalkGroup::Everbody);
    }
}
