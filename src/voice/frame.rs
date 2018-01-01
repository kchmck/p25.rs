//! Decode a voice frame into chunks suitable for IMBE.

use bits::Dibit;
use coding::{golay, hamming};
use consts;
use error::Result;

use voice::descramble::descramble;
use voice::rand;

use error::P25Error::*;

/// IMBE-encoded voice frame.
pub struct VoiceFrame {
    /// Chunks of IMBE-prioritized data, Known as `u_0`, ..., `u_7` in the standard.
    pub chunks: [u32; 8],
    /// Number of FEC errors detected for each associated chunk `u_0`, ..., `u_6`.
    pub errors: [usize; 7],
}

impl VoiceFrame {
    /// Try to decode a `VoiceFrame` from the given coded, PN-scrambled, interleaved
    /// dibits. Return `Ok(frame)` if the frame was successfully decoded, and `Err(err)`
    /// otherwise.
    pub fn new(dibits: &[Dibit; consts::FRAME_DIBITS]) -> Result<VoiceFrame> {
        let mut chunks = [0; 8];
        let mut errors = [0; 7];

        // Decode u_0 to recover the PN seed.
        let (init, err) = match golay::standard::decode(descramble(dibits, 0)) {
            Some(x) => x,
            None => return Err(GolayStdUnrecoverable),
        };

        let mut prand = rand::PseudoRand::new(init);

        chunks[0] = init as u32;
        errors[0] = err;

        // Decode "higher-priority" Golay chunks.
        for idx in 1..=3 {
            let bits = descramble(dibits, idx) ^ prand.next_23();

            let (data, err) = match golay::standard::decode(bits) {
                Some(x) => x,
                None => return Err(GolayStdUnrecoverable),
            };

            errors[idx] = err;
            chunks[idx] = data as u32;
        }

        // Decode "lower-priority" Hamming chunks.
        for idx in 4..=6 {
            let bits = descramble(dibits, idx) ^ prand.next_15();

            let (data, err) = match hamming::standard::decode(bits as u16) {
                Some(x) => x,
                None => return Err(HammingStdUnrecoverable),
            };

            errors[idx] = err;
            chunks[idx] = data as u32;
        }

        chunks[7] = descramble(dibits, 7) as u32;

        Ok(VoiceFrame {
            chunks: chunks,
            errors: errors,
        })
    }
}
