//! Encoding and decoding of the (23, 12, 7) standard, (24, 12, 8) extended, and (18, 6,
//! 8) shortened Golay codes described by P25.

pub use cai_golay::{standard, extended};

/// Encoding and decoding of the (18, 6, 8) code.
pub mod shortened {
    use super::*;

    /// Encode the given 6 data bits to an 18-bit codeword.
    pub fn encode(data: u8) -> u32 {
        assert_eq!(data >> 6, 0);
        extended::encode(data as u16)
    }

    /// Try to decode the given 18-bit word to the nearest codeword, correcting up to 3
    /// errors.
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 6
    /// data bits and `err` is the number of corrected bits. Otherwise, return `None` to
    /// indicate an unrecoverable error.
    pub fn decode(word: u32) -> Option<(u8, usize)> {
        assert_eq!(word >> 18, 0);

        extended::decode(word)
            .and_then(|(data, err)| if data >> 6 == 0 {
                Some((data as u8, err))
            } else {
                None
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shortened() {
        assert_eq!(shortened::encode(0), 0);
        assert_eq!(shortened::encode(0b111111), 0b111111_001100101110);
        assert_eq!(shortened::encode(0b000111), 0b000111_101101000010);
        assert_eq!(shortened::encode(0b111000), 0b111000_100001101100);
        assert_eq!(shortened::encode(0b100001), 0b100001_111000100110);

        let w = 0b101010;
        let e = shortened::encode(w);
        assert_eq!(e, 0b101010_001000110101);

        assert_eq!(shortened::decode(e^0b100000000000000001), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b010000000000000010), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b001000000000000100), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000100000000001000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000010000000010000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000001000000100000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000100001000000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000010010000000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000001100000000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000000101000000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000001000100000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000010000010000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000100000001000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000001000000000100), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000010000000000010), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000100000000000001), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b001000000000000100), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b010000000000010000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b100000000000100000), Some((w, 2)));

        assert_eq!(shortened::decode(e^0b111000000000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b011100000000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b001110000000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000111000000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000011100000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000001110000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000111000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000011100000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000001110000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000111000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000011100000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000001110000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000000111000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000000011100), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000000001110), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000000000000000111), Some((w, 3)));

        assert_eq!(shortened::decode(e^0b000000000000000000), Some((w, 0)));
        assert_eq!(shortened::decode(e^0b000000000000000001), Some((w, 1)));
        assert_eq!(shortened::decode(e^0b000000000000000011), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b000000000000000111), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b000100000000000000), Some((w, 1)));
        assert_eq!(shortened::decode(e^0b001100000000000000), Some((w, 2)));
        assert_eq!(shortened::decode(e^0b011100000000000000), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b001100000000000010), Some((w, 3)));
        assert_eq!(shortened::decode(e^0b001000000000000110), Some((w, 3)));
    }
}
