use collect_slice::CollectSlice;

use bits::{Dibit, Hexbit, HexbitBytes};
use buffer::{Buffer, VoiceHeaderWordStorage, VoiceHeaderStorage};
use coding::{reed_solomon, golay};
use consts::HEADER_BYTES;
use error::Result;
use trunking::fields::TalkGroup;
use util::slice_u16;
use voice::crypto::CryptoAlgorithm;

use error::P25Error::*;

pub struct VoiceHeaderReceiver {
    dibits: Buffer<VoiceHeaderWordStorage>,
    hexbits: Buffer<VoiceHeaderStorage>,
}

impl VoiceHeaderReceiver {
    pub fn new() -> VoiceHeaderReceiver {
        VoiceHeaderReceiver {
            dibits: Buffer::new(VoiceHeaderWordStorage::new()),
            hexbits: Buffer::new(VoiceHeaderStorage::new()),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<VoiceHeaderFields>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u32,
            None => return None,
        };

        let data = match golay::shortened::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(GolayUnrecoverable)),
        };

        let hexbits = match self.hexbits.feed(Hexbit::new(data)) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match reed_solomon::long::decode(hexbits) {
            Some((data, err)) => data,
            None => return Some(Err(ReedSolomonUnrecoverable)),
        };

        let mut bytes = [0; HEADER_BYTES];
        HexbitBytes::new(data.iter().cloned())
            .collect_slice_checked(&mut bytes[..]);

        Some(Ok(VoiceHeaderFields::new(bytes)))
    }
}

pub struct VoiceHeaderFields([u8; HEADER_BYTES]);

impl VoiceHeaderFields {
    pub fn new(buf: [u8; HEADER_BYTES]) -> Self { VoiceHeaderFields(buf) }

    pub fn crypto_init(&self) -> &[u8] { &self.0[..9] }
    pub fn mfg(&self) -> u8 { self.0[9] }

    pub fn crypto_alg(&self) -> CryptoAlgorithm {
        CryptoAlgorithm::from_bits(self.0[10])
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
    use trunking::fields::TalkGroup;

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
