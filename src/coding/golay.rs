//! Encoding and decoding of the (23, 12, 7) standard, (24, 12, 8) extended, and (18, 6,
//! 8) shortened Golay codes described by P25.
//!
//! These algorithms are sourced from *Coding Theory and Cryptography: The Essentials*,
//! Hankerson, Hoffman, et al, 2000.

/// Encoding and decoding of the (23, 12, 7) code.
pub mod standard {
    /// Encode the given 12 data bits into a 23-bit codeword.
    pub fn encode(data: u16) -> u32 {
        assert!(data >> 12 == 0);
        matrix_mul_systematic!(data, super::CORE_SHORT, u32)
    }

    /// Try to decode the given 23-bit word to the nearest codeword, correcting up to 3
    /// errors.
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 12
    /// data bits and `err` is the number of bits corrected in the data bits. Otherwise,
    /// return `None` to indicate an unrecoverable error.
    pub fn decode(word: u32) -> Option<(u16, usize)> {
        assert!(word >> 23 == 0);

        // Create a 24-bit codeword with odd weight.
        let expanded = if word.count_ones() % 2 == 0 {
            word << 1 | 1
        } else {
            word << 1
        };

        let data = super::word_data(expanded);
        let s = super::syndrome_24(expanded);

        if s == *super::CORE.last().unwrap() {
            Some((data, 0))
        } else {
            super::decode_syndrome(data, s)
        }
    }
}

/// Encoding and decoding of the (24, 12, 8) code.
pub mod extended {
    /// Encode the given 12 data bits into a 24-bit codeword.
    pub fn encode(data: u16) -> u32 {
        assert!(data >> 12 == 0);
        matrix_mul_systematic!(data, super::CORE, u32)
    }

    /// Try to decode the given  24-bit word to the nearest codeword, correcting up to 3
    /// errors.
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 12
    /// data bits and `err` is the number of bits corrected in the data bits. Otherwise,
    /// return `None` to indicate an unrecoverable error.
    pub fn decode(word: u32) -> Option<(u16, usize)> {
        assert!(word >> 24 == 0);
        super::decode_syndrome(super::word_data(word), super::syndrome_24(word))
    }
}

/// Encoding and decoding of the (18, 6, 8) code.
pub mod shortened {
    use super::extended;

    /// Encode the given 6 data bits to an 18-bit codeword.
    pub fn encode(data: u8) -> u32 {
        assert!(data >> 6 == 0);
        extended::encode(data as u16)
    }

    /// Try to decode the given 18-bit word to the nearest codeword, correcting up to 3
    /// errors.
    ///
    /// If decoding was successful, return `Some((data, err))`, where `data` is the 6
    /// data bits and `err` is the number of bits corrected in the data bits. Otherwise,
    /// return `None` to indicate an unrecoverable error.
    pub fn decode(word: u32) -> Option<(u8, usize)> {
        assert!(word >> 18 == 0);

        match extended::decode(word) {
            Some((data, err)) => if data >> 6 != 0 {
                None
            } else {
                Some((data as u8, err))
            },
            None => None,
        }
    }
}

/// The core matrix used to create the generator and syndrome matrices. It's usually
/// cyclic, but not in the case of P25.
const CORE: [u16; 12] = [
    0b101001001111,
    0b111101101000,
    0b011110110100,
    0b001111011010,
    0b000111101101,
    0b101010111001,
    0b111100010011,
    0b110111000110,
    0b011011100011,
    0b100100111110,
    0b010010011111,
    0b110001110101,
];

/// The core matrix and its transpose are equal if it's cyclic, but that's not the case
/// here.
const CORE_XPOSE: [u16; 12] = [
    0b110001110101,
    0b011000111011,
    0b111101101000,
    0b011110110100,
    0b001111011010,
    0b110110011001,
    0b011011001101,
    0b001101100111,
    0b110111000110,
    0b101010010111,
    0b100100111110,
    0b100011101011,
];

/// The core matrix with the LSB of each row removed.
const CORE_SHORT: [u16; 11] = [
    0b101001001111,
    0b111101101000,
    0b011110110100,
    0b001111011010,
    0b000111101101,
    0b101010111001,
    0b111100010011,
    0b110111000110,
    0b011011100011,
    0b100100111110,
    0b010010011111,
];

/// Syndrome/parity-check matrix.
const PAR: [u32; 12] = [
    0b100000000000110001110101,
    0b010000000000011000111011,
    0b001000000000111101101000,
    0b000100000000011110110100,
    0b000010000000001111011010,
    0b000001000000110110011001,
    0b000000100000011011001101,
    0b000000010000001101100111,
    0b000000001000110111000110,
    0b000000000100101010010111,
    0b000000000010100100111110,
    0b000000000001100011101011,
];

/// Try to correct errors in the given data bits using the given first-level syndrome.
fn decode_syndrome(data: u16, s: u16) -> Option<(u16, usize)> {
    decode_parity(s, &CORE).map(|(a, _)| {
        (data ^ a, a.count_ones() as usize)
    }).or(decode_parity(syndrome_12(s), &CORE_XPOSE).map(|(_, b)| {
        (data ^ b, b.count_ones() as usize)
    }))
}

/// Try to find an error pattern for the given syndrome using the rows from the given
/// matrix.
fn decode_parity(s: u16, matrix: &[u16; 12]) -> Option<(u16, u16)> {
    if s.count_ones() <= 3 {
        return Some((s, 0));
    }

    for (i, sum) in matrix.iter().map(|row| s ^ row).enumerate() {
        if sum.count_ones() <= 2 {
            return Some((sum, 1 << (12 - i - 1)));
        }
    }

    None
}

/// Calculate the first-level syndrome.
fn syndrome_24(word: u32) -> u16 {
    matrix_mul!(word, PAR, u16)
}

/// Calculate the second-level syndrome.
fn syndrome_12(syn: u16) -> u16 {
    matrix_mul!(syn, CORE, u16)
}

/// Extract the data bits from the given codeword.
fn word_data(word: u32) -> u16 {
    (word >> 12) as u16
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shortened() {
        let w = 0b101010;
        let e = shortened::encode(w);

        assert_eq!(Some((w, 1)), shortened::decode(e^0b100000000000000001));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b010000000000000010));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b001000000000000100));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000100000000001000));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000010000000010000));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000001000000100000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000100001000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000010010000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000001100000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000101000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000001000100000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000010000010000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000100000001000));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000001000000000100));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000010000000000010));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000100000000000001));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b001000000000000100));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b010000000000010000));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b100000000000100000));

        assert_eq!(Some((w, 3)), shortened::decode(e^0b111000000000000000));
        assert_eq!(Some((w, 3)), shortened::decode(e^0b011100000000000000));
        assert_eq!(Some((w, 3)), shortened::decode(e^0b001110000000000000));
        assert_eq!(Some((w, 3)), shortened::decode(e^0b000111000000000000));
        assert_eq!(Some((w, 2)), shortened::decode(e^0b000011100000000000));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000001110000000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000111000000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000011100000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000001110000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000111000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000011100000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000001110000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000111000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000011100));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000001110));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000000111));

        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000000000));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000000001));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000000011));
        assert_eq!(Some((w, 0)), shortened::decode(e^0b000000000000000111));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b000100000000000000));
        assert_eq!(Some((w, 2)), shortened::decode(e^0b001100000000000000));
        assert_eq!(Some((w, 3)), shortened::decode(e^0b011100000000000000));
        assert_eq!(Some((w, 2)), shortened::decode(e^0b001100000000000010));
        assert_eq!(Some((w, 1)), shortened::decode(e^0b001000000000000110));

        for i in 0..1<<6 {
            assert_eq!(shortened::decode(shortened::encode(i)).unwrap().0, i);
        }
    }

    #[test]
    fn test_standard() {
        let w = 0b101010101010;
        let e = standard::encode(w);

        assert_eq!(Some((w, 1)), standard::decode(e^0b10000000000000000000001));
        assert_eq!(Some((w, 1)), standard::decode(e^0b01000000000000000000010));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00100000000000000000100));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00010000000000000001000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00001000000000000010000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000100000000000100000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000010000000001000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000001000000010000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000100000100000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000010001000000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000001010000000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000010000000000001));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000100000000000010));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000001000000000000100));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000010000000000001000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000100000000000010000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00001000000000000100000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00010000000000001000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00100000000000010000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b01000000000000100000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b10000000000001000000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b10000000000010000000000));

        assert_eq!(Some((w, 3)), standard::decode(e^0b11100000000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b01110000000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00111000000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00011100000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00001110000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000111000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000011100000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000001110000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000000111000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000000011100000000000));
        assert_eq!(Some((w, 2)), standard::decode(e^0b00000000001110000000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000000000111000000000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000011100000000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000001110000000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000111000000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000011100000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000001110000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000111000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000011100));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000001110));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000000111));

        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000000000));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000000001));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000000011));
        assert_eq!(Some((w, 0)), standard::decode(e^0b00000000000000000000111));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000001000000000000000));
        assert_eq!(Some((w, 2)), standard::decode(e^0b00000011000000000000000));
        assert_eq!(Some((w, 3)), standard::decode(e^0b00000111000000000000000));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000100000000000000001));
        assert_eq!(Some((w, 2)), standard::decode(e^0b00000110000000000000001));
        assert_eq!(Some((w, 1)), standard::decode(e^0b00000100000000000000011));

        for i in 0..1<<12 {
            assert_eq!(standard::decode(standard::encode(i)).unwrap().0, i);
        }
    }

    #[test]
    fn test_extended() {
        let w = 0b101010101010;
        let e = extended::encode(w);

        assert_eq!(Some((w, 1)), extended::decode(e^0b100000000000000000000010));
        assert_eq!(Some((w, 1)), extended::decode(e^0b010000000000000000000001));
        assert_eq!(Some((w, 1)), extended::decode(e^0b001000000000000000000010));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000100000000000000000100));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000010000000000000001000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000001000000000000010000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000100000000000100000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000010000000001000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000001000000010000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000100000100000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000010001000000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000001010000000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000010000000000001));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000100000000000010));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000001000000000000100));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000010000000000001000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000100000000000010000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000001000000000000100000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000010000000000001000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000100000000000010000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b001000000000000100000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b010000000000001000000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b010000000000010000000000));

        assert_eq!(Some((w, 3)), extended::decode(e^0b111000000000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b011100000000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b001110000000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000111000000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000011100000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000001110000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000000111000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000000011100000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000000001110000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000000000111000000000000));
        assert_eq!(Some((w, 2)), extended::decode(e^0b000000000011100000000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000000001110000000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000111000000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000011100000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000001110000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000111000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000011100000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000001110000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000111000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000011100));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000001110));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000000111));

        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000000000));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000000001));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000000011));
        assert_eq!(Some((w, 0)), extended::decode(e^0b000000000000000000000111));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000001000000000000000));
        assert_eq!(Some((w, 2)), extended::decode(e^0b000000011000000000000000));
        assert_eq!(Some((w, 3)), extended::decode(e^0b000000111000000000000000));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000100000000000000001));
        assert_eq!(Some((w, 2)), extended::decode(e^0b000000110000000000000001));
        assert_eq!(Some((w, 1)), extended::decode(e^0b000000100000000000000011));

        for i in 0..1<<12 {
            assert_eq!(extended::decode(extended::encode(i)).unwrap().0, i);
        }
    }
}
