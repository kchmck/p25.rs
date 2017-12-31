//! Encoding and decoding of the (16, 8, 5) shortened cyclic code described by P25.
//!
//! The key information that this code is shortened from a (17, 8, 5) code came from
//! "Standard APCO25 Physical Layer of the Radio Transmission Chain", Simon, 2014.

use cai_cyclic;

/// Encode the given 8 data bits into a 16-bit codeword.
pub fn encode(data: u8) -> u16 {
    cai_cyclic::encode(data as u16) as u16
}

/// Try to decode the given 16-bit word to the nearest codeword, correcting up to 2
/// errors.
///
/// If decoding was successful, return `Some((data, err))`, where `data` is the 8 data
/// bits and `err` is the number of corrected bits. Otherwise, return `None` to indicate
/// an unrecoverable error.
pub fn decode(word: u16) -> Option<(u8, usize)> {
    cai_cyclic::decode(word as u32).and_then(|(word, err)| if word >> 8 == 0 {
        Some((word as u8, err))
    } else {
        None
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode() {
        let w = 0b10101011;
        let e = encode(w);
        assert_eq!(e, 0b10101011_01111011);

        assert_eq!(Some((w, 0)), decode(e^0b0000000000000000));
        assert_eq!(Some((w, 2)), decode(e^0b1000000000000001));
        assert_eq!(Some((w, 1)), decode(e^0b0001000000000000));
        assert_eq!(Some((w, 2)), decode(e^0b0011000000000000));

        assert_eq!(Some((w, 1)), decode(e^0b1000000000000000));
        assert_eq!(Some((w, 1)), decode(e^0b0100000000000000));
        assert_eq!(Some((w, 2)), decode(e^0b0010000000000001));
        assert_eq!(Some((w, 2)), decode(e^0b0001000000000010));
        assert_eq!(Some((w, 2)), decode(e^0b0000100000000100));
        assert_eq!(Some((w, 2)), decode(e^0b0000010000001000));
        assert_eq!(Some((w, 2)), decode(e^0b0000001000010000));
        assert_eq!(Some((w, 2)), decode(e^0b0000000100100000));
        assert_eq!(Some((w, 2)), decode(e^0b0000000011000000));
        assert_eq!(Some((w, 2)), decode(e^0b0000000001010000));
        assert_eq!(Some((w, 2)), decode(e^0b0000000010001000));
        assert_eq!(Some((w, 2)), decode(e^0b0000000100000100));
        assert_eq!(Some((w, 2)), decode(e^0b0000001000000010));
        assert_eq!(Some((w, 2)), decode(e^0b0000010000000001));
        assert_eq!(Some((w, 1)), decode(e^0b0000100000000000));
        assert_eq!(Some((w, 1)), decode(e^0b0001000000000000));
        assert_eq!(Some((w, 2)), decode(e^0b0010000000000001));
        assert_eq!(Some((w, 2)), decode(e^0b0100000000000100));
        assert_eq!(Some((w, 2)), decode(e^0b1000000000001000));

        for w in 0..=(!0u8) {
            assert_eq!(decode(encode(w as u8)), Some((w, 0)));
        }
    }
}
