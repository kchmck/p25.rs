//! Descramble/Deinterleave the dibits in a received voice frame.

use bits::Dibit;
use consts;

/// Descramble a portion of the given voice frame dibits into the PN-scrambled, coded
/// chunk `u_{idx}`.
pub fn descramble(dibits: &[Dibit; consts::FRAME_DIBITS], idx: usize) -> u32 {
    DESCRAMBLERS[idx].descramble(dibits)
}

/// Set of descramblers for each associated chunk `u_0`, ..., `u_7`.
const DESCRAMBLERS: [VoiceFrameDescrambler; 8] = [
    VoiceFrameDescrambler(&[
        ZigZag::hi(0, 23),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(69, 1),
        ZigZag::lo(0, 22),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(66, 2),
        ZigZag::hi(1, 21),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(64, 3),
        ZigZag::lo(1, 20),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(61, 4),
        ZigZag::hi(2, 11),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(35, 13),
        ZigZag::lo(2, 2),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::lo(8, 15),
    ]),
    VoiceFrameDescrambler(&[
        ZigZag::hi(53, 7),
    ]),
];

/// Descrambles input dibits according to the enclosed zigzag sequences.
struct VoiceFrameDescrambler(&'static [ZigZag]);

impl VoiceFrameDescrambler {
    /// Descramble the given dibits into a chunk.
    pub fn descramble(&self, dibits: &[Dibit; consts::FRAME_DIBITS]) -> u32 {
        // Zigzag results are concatenated.
        self.0.iter().fold(0, |word, zz| {
            zz.fold(word, |buf, (idx, hi)| {
                buf << 1 | if hi {
                    dibits[idx].hi() as u32
                } else {
                    dibits[idx].lo() as u32
                }
            })
        })
    }
}

/// Walks the zigzagging interleave schedule used for voice frames.
#[derive(Clone, Copy)]
struct ZigZag {
    /// Whether to use the high/MSB or low/LSB bit of the current dibit.
    hi: bool,
    /// Current dibit index into the packet.
    idx: usize,
    /// Number of symbols remaining in this zigzag.
    remain: usize,
}

impl ZigZag {
    /// Start a zigzag at the high bit of the given index to decode the given number of
    /// bits.
    pub const fn hi(start: usize, num: usize) -> ZigZag {
        ZigZag {
            hi: true,
            idx: start,
            remain: num,
        }
    }

    /// Start a zigzag at the low bit of the given index to decode the given number of
    /// bits.
    pub const fn lo(start: usize, num: usize) -> ZigZag {
        ZigZag {
            hi: false,
            idx: start,
            remain: num,
        }
    }
}

impl Iterator for ZigZag {
    type Item = (usize, bool);

    /// If the zigzag isn't exhausted, go to the next dibit in the sequence and return
    /// `Some((idx, hi))`, where `idx` is its index in the packet, and `hi` indicates
    /// whether to use its high or low bit. Otherwise, return `None`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.remain == 0 {
            return None;
        }

        let cur = (self.idx, self.hi);

        self.idx += 3;
        self.remain -= 1;
        self.hi = !self.hi;

        Some(cur)
    }
}

#[cfg(test)]
mod test {
    use super::DESCRAMBLERS;
    use consts;

    #[test]
    fn test_steps_exhaustive() {
        let mut visited = [0u32; consts::FRAME_DIBITS];

        for d in DESCRAMBLERS.iter() {
            for &zz in d.0.iter() {
                for (idx, _) in zz {
                    visited[idx] += 1;
                }
            }
        }

        for &v in visited.iter() {
            assert_eq!(v, 2);
        }
    }
}
