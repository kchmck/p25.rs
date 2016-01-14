use bits::Dibit;
use voice::consts;

pub fn descramble(dibits: &[Dibit; consts::FRAME_DIBITS], idx: usize) -> u32 {
    DESCRAMBLERS[idx].descramble(dibits)
}

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
        ZigZag::lo(1, 20),
        ZigZag::lo(64, 3),
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

struct VoiceFrameDescrambler(&'static [ZigZag]);

impl VoiceFrameDescrambler {
    pub fn descramble(&self, dibits: &[Dibit; consts::FRAME_DIBITS]) -> u32 {
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

#[derive(Clone, Copy)]
struct ZigZag {
    hi: bool,
    idx: usize,
    num: usize,
}

impl ZigZag {
    pub const fn hi(start: usize, num: usize) -> ZigZag {
        ZigZag {
            hi: true,
            idx: start,
            num: num,
        }
    }

    pub const fn lo(start: usize, num: usize) -> ZigZag {
        ZigZag {
            hi: false,
            idx: start,
            num: num,
        }
    }
}

impl Iterator for ZigZag {
    type Item = (usize, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.num == 0 {
            return None;
        }

        let cur = (self.idx, self.hi);

        self.idx += 3;
        self.num -= 1;
        self.hi = !self.hi;

        Some(cur)
    }
}

#[cfg(test)]
mod test {
    use super::DESCRAMBLERS;
    use voice::consts;

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
