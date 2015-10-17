//! This module implements encoding and decoding of the (63, 16, 23) BCH code used to
//! protect P25's NID field.
//!
//! It uses an optimized "matrix multiplication" for encoding and
//! the Berlekamp-Massey algorithm followed by Chien search for decoding, and both use
//! only stack memory.
//!
//! Most Galois field information as well as the Berlekamp-Massey implementation are
//! derived from \[1] and the Chien search was derived from \[2].
//!
//! \[1]: "Coding Theory and Cryptography: The Essentials", 2nd ed, Hankerson, Hoffman, et
//! al, 2000
//!
//! \[2]: https://en.wikipedia.org/wiki/Chien_search

use std;

use galois::{GaloisField, P25Field, P25Codeword, Polynomial, PolynomialCoefs};
use bmcf;

/// Encode the given word into a P25 BCH codeword.
pub fn encode(word: u16) -> u64 {
    matrix_mul_systematic!(word, GEN, u64)
}

/// Decode the given codeword into data bits, correcting up to 11 errors. Return
/// `Some((data, err))`, where `data` is the data bits and `err` is the number of errors,
/// if the codeword could be corrected and `None` if it couldn't.
pub fn decode(word: u64) -> Option<(u16, usize)> {
    // The BCH code is only over the first 63 bits, so strip off the P25 parity bit.
    let word = word >> 1;

    let syn = syndromes(word);
    // Get the error location polynomial.
    let poly = BCHDecoder::new(syn).decode();

    // The degree indicates the number of errors that need to be corrected.
    let errors = match poly.degree() {
        Some(deg) => deg,
        None => panic!("invalid polynomial"),
    };

    // Even if there are more errors, the BM algorithm produces a polynomial with degree
    // no greater than ERRORS.
    assert!(errors <= BCHCoefs::errors());

    // Get the bit locations from the polynomial.
    let locs = bmcf::Errors::new(poly, syn);

    // Correct the codeword and count the number of corrected errors. Stop the
    // `Errors` iteration after `errors` iterations since it won't yield any more
    // locations after that anyway.
    let (word, count) = locs.take(errors).fold((word, 0), |(w, s), (loc, val)| {
        assert!(val.power().unwrap() == 0);
        (w ^ 1 << loc, s + 1)
    });

    if count == errors {
        // Strip off the (corrected) parity-check bits.
        Some(((word >> 47) as u16, errors))
    } else {
        None
    }
}

/// Generator matrix from P25, transformed for more efficient codeword generation.
const GEN: &'static [u16] = &[
    0b1110110001000111,
    0b1001101001100100,
    0b0100110100110010,
    0b0010011010011001,
    0b1111111100001011,
    0b1001001111000010,
    0b0100100111100001,
    0b1100100010110111,
    0b1000100000011100,
    0b0100010000001110,
    0b0010001000000111,
    0b1111110101000100,
    0b0111111010100010,
    0b0011111101010001,
    0b1111001111101111,
    0b1001010110110000,
    0b0100101011011000,
    0b0010010101101100,
    0b0001001010110110,
    0b0000100101011011,
    0b1110100011101010,
    0b0111010001110101,
    0b1101011001111101,
    0b1000011101111001,
    0b1010111111111011,
    0b1011101110111010,
    0b0101110111011101,
    0b1100001010101001,
    0b1000110100010011,
    0b1010101011001110,
    0b0101010101100111,
    0b1100011011110100,
    0b0110001101111010,
    0b0011000110111101,
    0b1111010010011001,
    0b1001011000001011,
    0b1010011101000010,
    0b0101001110100001,
    0b1100010110010111,
    0b1000111010001100,
    0b0100011101000110,
    0b0010001110100011,
    0b1111110110010110,
    0b0111111011001011,
    0b1101001100100010,
    0b0110100110010001,
    0b1101100010001111,
    0b0000000000000011,
];

#[derive(Copy, Clone, Default)]
struct BCHCoefs([P25Codeword; 22 + 2]);

impl std::ops::Deref for BCHCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl std::ops::DerefMut for BCHCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

impl PolynomialCoefs for BCHCoefs {
    fn distance() -> usize { 23 }
}

type BCHPolynomial = Polynomial<BCHCoefs>;
type BCHDecoder = bmcf::BerlMasseyDecoder<BCHCoefs>;

/// Generate the syndrome polynomial for the given received word.
fn syndromes(word: u64) -> BCHPolynomial {
    BCHPolynomial::new(std::iter::once(P25Codeword::for_power(0))
        .chain((1..BCHCoefs::distance()).map(|pow| {
            (0..P25Field::size()).fold(P25Codeword::default(), |s, b| {
                if word >> b & 1 == 0 {
                    s
                } else {
                    s + P25Codeword::for_power(b * pow)
                }
            })
        }))
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{syndromes, BCHDecoder, BCHCoefs, BCHPolynomial};
    use galois::{P25Codeword, PolynomialCoefs};
    use bmcf::Errors;

    #[test]
    fn validate_coefs() {
        BCHCoefs::default().validate();
    }

    #[test]
    fn test_encode() {
        assert_eq!(encode(0b1111111100000000), 0b1111111100000000100100110001000011000010001100000110100001101000);
        assert_eq!(encode(0b0011)&1, 0);
        assert_eq!(encode(0b0101)&1, 1);
        assert_eq!(encode(0b1010)&1, 1);
        assert_eq!(encode(0b1100)&1, 0);
        assert_eq!(encode(0b1111)&1, 0);
    }

    #[test]
    fn test_syndromes() {
        let w = encode(0b1111111100000000)>>1;
        let p = syndromes(w);

        assert_eq!(p.len(), 24);
        assert_eq!(p.degree().unwrap(), 0);
        assert_eq!(syndromes(w ^ 1<<60).degree().unwrap(), 22);
        assert_eq!(p[0], P25Codeword::for_power(0));
    }

    #[test]
    fn test_decoder() {
        let w = encode(0b1111111100000000)^0b11<<61;
        let poly = BCHDecoder::new(syndromes(w >> 1)).decode();

        assert!(poly.coef(0).power().unwrap() == 0);
        assert!(poly.coef(1).power().unwrap() == 3);
        assert!(poly.coef(2).power().unwrap() == 58);
    }

    #[test]
    fn test_locs() {
        let w = encode(0b0000111100001111)^0b11<<61;
        let p = syndromes(w >> 1);

        let coefs = BCHPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned());

        let mut locs = Errors::new(coefs, p);

        assert_eq!(locs.next().unwrap(), (61, P25Codeword::for_power(0)));
        assert_eq!(locs.next().unwrap(), (60, P25Codeword::for_power(0)));
        assert!(locs.next().is_none());
    }

    #[test]
    fn test_decode() {
        assert!(decode(encode(0b0000111100001111) ^ 1<<63).unwrap() ==
                (0b0000111100001111, 1));

        assert!(decode(encode(0b1100011111111111) ^ 1).unwrap() ==
                (0b1100011111111111, 0));

        assert!(decode(encode(0b1111111100000000) ^ 0b11010011<<30).unwrap() ==
                (0b1111111100000000, 5));

        assert!(decode(encode(0b1101101101010001) ^ (1<<63 | 1)).unwrap() ==
                (0b1101101101010001, 1));

        assert!(decode(encode(0b1111111111111111) ^ 0b11111111111).unwrap() ==
                (0b1111111111111111, 10));

        assert!(decode(encode(0b0000000000000000) ^ 0b11111111111).unwrap() ==
                (0b0000000000000000, 10));

        assert!(decode(encode(0b0000111110000000) ^ 0b111111111110).unwrap() ==
                (0b0000111110000000, 11));

        assert!(decode(encode(0b0000111110000000) ^ 0b111111111110).unwrap() ==
                (0b0000111110000000, 11));

        assert!(decode(encode(0b0000111110001010) ^ 0b1111111111110).is_none());
        assert!(decode(encode(0b0000001111111111) ^ 0b11111111111111111111110).is_none());
        assert!(decode(encode(0b0000001111111111) ^
                       0b00100101010101000010001100100010011111111110).is_none());
    }
}
