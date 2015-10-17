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
    // Get the error location polynomial.
    let poly = BCHDecoder::new(Syndromes::new(word).poly()).decode();

    // The degree indicates the number of errors that need to be corrected.
    let errors = match poly.degree() {
        Some(deg) => deg,
        None => panic!("invalid polynomial"),
    };

    // Even if there are more errors, the BM algorithm produces a polynomial with degree
    // no greater than ERRORS.
    assert!(errors <= ERRORS);

    // Get the bit locations from the polynomial.
    let locs = ErrorLocations::new(poly.iter().cloned());

    // Correct the codeword and count the number of corrected errors. Stop the
    // `ErrorLocations` iteration after `errors` iterations since it won't yield any more
    // locations after that anyway.
    let (word, count) = locs.take(errors).fold((word, 0), |(w, s), loc| {
        (w ^ 1 << loc, s + 1)
    });

    if count == errors {
        // Strip off the (corrected) parity-check bits.
        Some(((word >> 47) as u16, errors))
    } else {
        None
    }
}

/// The d in (n,k,d).
const DISTANCE: usize = 23;
/// 2t+1 = 23 => t = 11
const ERRORS: usize = 11;
/// Required syndrome codewords.
const SYNDROMES: usize = 2 * ERRORS;

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
struct BCHCoefs([P25Codeword; SYNDROMES + 2]);

impl std::ops::Deref for BCHCoefs {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.0[..] }
}

impl std::ops::DerefMut for BCHCoefs {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
}

impl PolynomialCoefs for BCHCoefs {}

type BCHPolynomial = Polynomial<BCHCoefs>;

/// Iterator over the syndromes of a received codeword. Each syndrome is a codeword in
/// GF(2^6).
struct Syndromes {
    /// Exponent power of the current syndrome.
    pow: std::ops::Range<usize>,
    /// Received codeword itself.
    word: u64,
}

impl Syndromes {
    /// Construct a new `Syndromes` for the given received codeword.
    pub fn new(word: u64) -> Syndromes {
        Syndromes {
            pow: 1..DISTANCE,
            word: word,
        }
    }

    /// Generate the "syndrome polynomial".
    pub fn poly(mut self) -> BCHPolynomial {
        BCHPolynomial::new(std::iter::once(P25Codeword::for_power(0))
                                    .chain(self.into_iter()))
    }
}

impl Iterator for Syndromes {
    type Item = P25Codeword;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pow.next() {
            Some(pow) => Some((0..P25Field::size()).fold(P25Codeword::default(), |s, b| {
                if self.word >> b & 1 == 0 {
                    s
                } else {
                    s + P25Codeword::for_power(b * pow)
                }
            })),
            None => None,
        }
    }
}

/// Implements the iterative part of the Berlekamp-Massey algorithm.
struct BerlMasseyDecoder<P: PolynomialCoefs> {
    /// Saved p polynomial: p_{z_i-1}.
    p_saved: Polynomial<P>,
    /// Previous iteration's p polynomial: p_{i-1}.
    p_cur: Polynomial<P>,
    /// Saved q polynomial: q_{z_i-1}.
    q_saved: Polynomial<P>,
    /// Previous iteration's q polynomial: q_{i-1}.
    q_cur: Polynomial<P>,
    /// Degree-related term of saved p polynomial: D_{z_i-1}.
    deg_saved: usize,
    /// Degree-related term of previous p polynomial: D_{i-1}.
    deg_cur: usize,
}

impl<P: PolynomialCoefs> BerlMasseyDecoder<P> {
    /// Construct a new `BerlMasseyDecoder` from the given syndrome codeword iterator.
    pub fn new(syndromes: Polynomial<P>) -> BerlMasseyDecoder<P> {
        // 2t zeroes followed by a one.
        let p = Polynomial::new((0..SYNDROMES+1).map(|_| P25Codeword::default())
                                    .chain(std::iter::once(P25Codeword::for_power(0))));

        BerlMasseyDecoder {
            q_saved: syndromes,
            q_cur: syndromes.shift(),
            p_saved: p,
            p_cur: p.shift(),
            deg_saved: 0,
            deg_cur: 1,
        }
    }

    /// Perform the iterative steps to get the error-location polynomial Λ(x) wih deg(Λ)
    /// <= t.
    pub fn decode(mut self) -> Polynomial<P> {
        for _ in 0..SYNDROMES {
            self.step();
        }

        self.p_cur
    }

    /// Perform one iterative step of the algorithm, updating the state polynomials and
    /// degrees.
    fn step(&mut self) {
        let (save, q, p, d) = if self.q_cur.constant().zero() {
            self.reduce()
        } else {
            self.transform()
        };

        if save {
            self.q_saved = self.q_cur;
            self.p_saved = self.p_cur;
            self.deg_saved = self.deg_cur;
        }

        self.q_cur = q;
        self.p_cur = p;
        self.deg_cur = d;
    }

    /// Simply shift the polynomials since they have no degree-0 term.
    fn reduce(&mut self) -> (bool, Polynomial<P>, Polynomial<P>, usize) {
        (
            false,
            self.q_cur.shift(),
            self.p_cur.shift(),
            2 + self.deg_cur,
        )
    }

    /// Remove the degree-0 terms and shift the polynomials.
    fn transform(&mut self) -> (bool, Polynomial<P>, Polynomial<P>, usize) {
        let mult = self.q_cur.constant() / self.q_saved.constant();

        (
            self.deg_cur >= self.deg_saved,
            (self.q_cur + self.q_saved * mult).shift(),
            (self.p_cur + self.p_saved * mult).shift(),
            2 + std::cmp::min(self.deg_cur, self.deg_saved),
        )
   }
}

type BCHDecoder = BerlMasseyDecoder<BCHCoefs>;

/// Uses Chien search to find the roots in GF(2^6) of an error-locator polynomial and
/// produce an iterator of error bit positions.
struct ErrorLocations {
    /// Coefficients of the polynomial.
    terms: [P25Codeword; ERRORS + 1],
    /// Current exponent power of the iteration.
    pow: std::ops::Range<usize>,
}

impl ErrorLocations {
    /// Construct a new `ErrorLocations` from the given coefficients, where Λ(x) =
    /// coefs[0] + coefs[1]*x + ... + coefs[e]*x^e.
    pub fn new<T: Iterator<Item = P25Codeword>>(coefs: T) -> ErrorLocations {
        // The maximum degree is t error locations (t+1 coefficients.)
        let mut poly = [P25Codeword::default(); ERRORS + 1];

        for (pow, (cur, coef)) in poly.iter_mut().zip(coefs).enumerate() {
            // Since the first call to `update_terms()` multiplies by `pow` and the
            // coefficients should equal themselves on the first iteration, divide by
            // `pow` here.
            *cur = *cur + coef / P25Codeword::for_power(pow)
        }

        ErrorLocations {
            terms: poly,
            pow: 0..P25Field::size(),
        }
    }

    /// Perform the term-updating step of the algorithm: x_{j,i} = x_{j,i-1} * α^j.
    fn update_terms(&mut self) {
        for (pow, term) in self.terms.iter_mut().enumerate() {
            *term = *term * P25Codeword::for_power(pow);
        }
    }

    /// Calculate the sum of the terms: x_{0,i} + x_{1,i} + ... + x_{t,i} -- evaluate the
    /// error-locator polynomial at Λ(α^i).
    fn sum_terms(&self) -> P25Codeword {
        self.terms.iter().fold(P25Codeword::default(), |s, &x| s + x)
    }
}

impl Iterator for ErrorLocations {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let pow = match self.pow.next() {
                Some(pow) => pow,
                None => return None,
            };

            self.update_terms();

            if self.sum_terms().zero() {
                return Some(P25Codeword::for_power(pow).invert().power().unwrap());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{Syndromes, BCHDecoder, ErrorLocations};
    use galois::P25Codeword;

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

        assert!(Syndromes::new(w).all(|s| s.zero()));
        assert!(!Syndromes::new(w ^ 1<<60).all(|s| s.zero()));
    }

    #[test]
    fn test_decoder() {
        let w = encode(0b1111111100000000)^0b11<<61;
        let poly = BCHDecoder::new(Syndromes::new(w >> 1).poly()).decode();

        assert!(poly.coef(0).power().unwrap() == 0);
        assert!(poly.coef(1).power().unwrap() == 3);
        assert!(poly.coef(2).power().unwrap() == 58);
    }

    #[test]
    fn test_locs() {
        let coefs = [P25Codeword::for_power(0), P25Codeword::for_power(3),
                     P25Codeword::for_power(58)];
        let mut locs = ErrorLocations::new(coefs.iter().cloned());

        assert!(locs.next().unwrap() == 61);
        assert!(locs.next().unwrap() == 60);
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
