//! Receive and decode voice header packets.

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

/// State machine for receiving a voice header packet.
pub struct VoiceHeaderReceiver {
    /// Current buffered dibits for the current hexbit.
    dibits: Buffer<VoiceHeaderWordStorage>,
    /// Current buffered hexbits.
    hexbits: Buffer<VoiceHeaderStorage>,
}

impl VoiceHeaderReceiver {
    /// Create a new `VoiceHeaderReceiver` in the initial state.
    pub fn new() -> VoiceHeaderReceiver {
        VoiceHeaderReceiver {
            dibits: Buffer::new(VoiceHeaderWordStorage::new()),
            hexbits: Buffer::new(VoiceHeaderStorage::new()),
        }
    }

    /// Feed in a baseband symbol, possibly producing a voice header packet. Return
    /// `Some(Ok(pkt))` if the packet was successfully received, `Some(Err(err))` if an
    /// error occurred, and `None` in the case of no event.
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

/// Buffer of bytes that represents a voice header packet.
pub type Buf = [u8; HEADER_BYTES];

/// Begins each voice message with information necessary to decode the following
/// superframes.
pub struct VoiceHeaderFields(Buf);

impl VoiceHeaderFields {
    /// Create a new `VoiceHeaderFields` decoder from the given bytes.
    pub fn new(buf: Buf) -> Self { VoiceHeaderFields(buf) }

    /// Initialization vector for cryptographic algorithm.
    pub fn crypto_init(&self) -> &[u8] { &self.0[..9] }
    /// Manufacturer ID.
    pub fn mfg(&self) -> u8 { self.0[9] }
    /// Cryptographic algorithm in use, if any.
    pub fn crypto_alg(&self) -> CryptoAlgorithm { CryptoAlgorithm::from_bits(self.0[10]) }
    /// Encryption key to use.
    pub fn crypto_key(&self) -> u16 { slice_u16(&self.0[11..]) }

    /// Talkgroup participating in the voice message.
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
