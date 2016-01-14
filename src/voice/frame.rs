use bits::Dibit;
use coding::{golay, hamming};
use error::Result;

use voice::descramble::descramble;
use voice::{consts, rand};

use error::P25Error::*;

pub struct VoiceFrame {
    pub chunks: [u32; 8],
    pub errors: [usize; 7],
}

impl VoiceFrame {
    pub fn new(dibits: &[Dibit; consts::FRAME_DIBITS]) -> Result<VoiceFrame> {
        let mut chunks = [0; 8];
        let mut errors = [0; 7];

        let (init, err) = match golay::standard::decode(descramble(dibits, 0)) {
            Some(x) => x,
            None => return Err(GolayUnrecoverable),
        };

        let mut prand = rand::PseudoRand::new(init);

        chunks[0] = init as u32;
        errors[0] = err;

        for idx in 1..4 {
            let bits = descramble(dibits, idx) ^ prand.next_23();

            let (data, err) = match golay::standard::decode(bits) {
                Some(x) => x,
                None => return Err(GolayUnrecoverable),
            };

            errors[idx] = err;
            chunks[idx] = data as u32;
        }

        for idx in 4..7 {
            let bits = descramble(dibits, idx) ^ prand.next_15();

            let (data, err) = match hamming::standard::decode(bits as u16) {
                Some(x) => x,
                None => return Err(HammingUnrecoverable),
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
