//! Encoding and decoding of the (63, 16, 23) BCH code described by P25.
//!
//! These algorithms are derived from *Coding Theory and Cryptography: The Essentials*,
//! Hankerson, Hoffman, et al, 2000.

use std;

use binfield_matrix::matrix_mul_systematic;

use coding::galois::{GaloisField, P25Field, P25Codeword, Polynomial, PolynomialCoefs};
use coding::bmcf;

/// Encode the given 16 data bits into a 64-bit codeword.
pub fn encode(word: u16) -> u64 {
    matrix_mul_systematic(word, GEN)
}

/// Try to decode the given 64-bit word to the nearest codeword, correcting up to 11
/// bit errors.
///
/// If decoding was successful, return `Some((data, err))`, where `data` is the 16 data
/// bits and `err` is the number of bits corrected. Otherwise, return `None` to indicate
/// an unrecoverable error.
pub fn decode(bits: u64) -> Option<(u16, usize)> {
    // The BCH code is only over the first 63 bits, so strip off the P25 parity bit.
    let word = bits >> 1;

    bmcf::Errors::new(syndromes(word)).map(|(nerr, errs)| {
        // Flip all error bits.
        let fixed = errs.fold(word, |w, (loc, pat)| {
            assert!(pat.power().unwrap() == 0);
            w ^ 1 << loc
        });

        // Strip off the parity bits.
        ((fixed >> 47) as u16, nerr)
    })
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

/// Polynomial coefficients for BCH decoding.
impl_polynomial_coefs!(BchCoefs, 23);

/// Polynomial with BCH coefficients.
type BchPolynomial = Polynomial<BchCoefs>;

/// Generate the syndrome polynomial s(x) from the given received word r(x).
///
/// The resulting polynomial has the form s(x) = s<sub>1</sub> + s<sub>2</sub>x + ··· +
/// s<sub>2t</sub>x<sup>2t</sup>, where s<sub>i</sub> = r(α<sup>i</sup>).
fn syndromes(word: u64) -> BchPolynomial {
    BchPolynomial::new((1..=BchCoefs::syndromes()).map(|p| {
        // Compute r(α^p) with the polynomial representation of the bitmap. The LSB of
        // `word` maps to the coefficient of the degree-0 term.
        (0..P25Field::size()).fold(P25Codeword::default(), |s, b| {
            if word >> b & 1 == 0 {
                s
            } else {
                s + P25Codeword::for_power(b * p)
            }
        })
    }))
}

#[cfg(test)]
mod test {
    use std;
    use super::*;
    use super::{syndromes, BchCoefs};
    use coding::galois::{PolynomialCoefs, P25Codeword, Polynomial};

    impl_polynomial_coefs!(TestCoefs, 23, 50);
    type TestPolynomial = Polynomial<TestCoefs>;

    #[test]
    fn validate_coefs() {
        BchCoefs::default().validate();
    }

    #[test]
    fn verify_gen() {
        // Verify construction of BCH generator polynomial g(x).

        let p = Polynomial::<TestCoefs>::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned()) * Polynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned());

        assert_eq!(p.degree().unwrap(), 47);

        let gen = 0o6331_1413_6723_5453u64;

        for i in 0..=47 {
            let coef = gen >> i & 1;

            assert_eq!(p[i].power(), if coef == 0 {
                None
            } else {
                Some(0)
            });
        }
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

        assert_eq!(syndromes(w).degree(), None);
        assert_eq!(syndromes(w ^ 1<<60).degree().unwrap(), 21);
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

        for i in 0..1u32<<17 {
            assert_eq!(decode(encode(i as u16)).unwrap().0, i as u16);
        }
    }
}
