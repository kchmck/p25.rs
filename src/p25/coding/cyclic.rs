//! Encoding and decoding of the (16, 8, 5) shortened cyclic code described by P25.
//!
//! The parity-check matrix construction was sourced from \[1] and the decoding algorithm
//! was derived from \[2]. The key information that this code is shortened from a (17, 8,
//! 5) code came from \[3].
//!
//! \[1]: "Coding Theory and Cryptography: The Essentials", 2nd ed, Hankerson, Hoffman, et
//! al, 2000
//! \[2]: "Error Control Coding", Lin and Costello, 1983
//! \[3]: "Standard APCO25 Physical Layer of the Radio Transmission Chain", Simon, 2014

/// Encode the 8 data bits into a 16-bit codeword.
pub fn encode(data: u8) -> u16 {
    matrix_mul_systematic!(data, GEN, u16)
}

/// Try to decode the 16-bit word to the nearest codeword, correcting up to 2 errors.
/// Return `Some((data, err))`, where `data` are the 12 data bits and `err` is the number
/// of errors corrected, on success and `None` on a detected unrecoverable error.
pub fn decode(word: u16) -> Option<(u8, usize)> {
    // Go through a full cycle of the codeword, so the data bits end up in their original
    // position. The word is expanded to 32 bits so it can be treated as the 17-bit word
    // the shortened code is derived from.
    let (fixed, word) = (0..17).fold((Some(0), word as u32), |(fixed, word), _| {
        let syn = syndrome(word);

        if syn == 0 {
            return (fixed, rotate_17(word));
        }

        match pattern(syn) {
            Some(pat) => (Some(pat.count_ones()), rotate_17(word ^ pat)),
            None => (None, rotate_17(word)),
        }
    });

    match fixed {
        Some(err) => Some(((word >> 8) as u8, err as usize)),
        None => None,
    }
}

/// Transposed generator matrix.
const GEN: [u8; 8] = [
    0b00111100,
    0b10011110,
    0b01001111,
    0b00011011,
    0b10110001,
    0b11100100,
    0b11110010,
    0b01111001,
];

/// Transposed parity-check matrix, where the rows of the original are generated from x^i
/// mod g(x).
const PAR: [u32; 8] = [
    0b10000000100111100,
    0b01000000010011110,
    0b00100000001001111,
    0b00010000100011011,
    0b00001000110110001,
    0b00000100111100100,
    0b00000010011110010,
    0b00000001001111001,
];

/// Calculate the syndrome of the given word.
fn syndrome(word: u32) -> u8 {
    matrix_mul!(word, PAR, u8)
}

/// Find the error pattern associated with the syndrome.
///
/// One of the benefits of the cyclic algorithm is we only have to store error patters
/// with the LSB set.
fn pattern(syn: u8) -> Option<u32> {
    match syn {
        0b00011001 => Some(0b00100000000000001),
        0b00011110 => Some(0b00000000001000001),
        0b00101001 => Some(0b00010000000000001),
        0b00110001 => Some(0b00001000000000001),
        0b00111000 => Some(0b00000001000000001),
        0b00111001 => Some(0b00000000000000001),
        0b00111011 => Some(0b00000010000000001),
        0b00111101 => Some(0b00000100000000001),
        0b01001011 => Some(0b00000000000000011),
        0b01110111 => Some(0b00000000010000001),
        0b01111001 => Some(0b01000000000000001),
        0b10100101 => Some(0b00000000100000001),
        0b10110110 => Some(0b00000000000100001),
        0b10111001 => Some(0b10000000000000001),
        0b11001000 => Some(0b00000000000001001),
        0b11011101 => Some(0b00000000000000101),
        0b11100010 => Some(0b00000000000010001),
        _ => None,
    }
}

/// Cyclically rotate the word as if it was 17 bits long.
fn rotate_17(word: u32) -> u32 {
    let lsb = word & 1;
    word >> 1 | lsb << 16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode() {
        let w = 0b10101011;
        let e = encode(w);

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
    }
}
