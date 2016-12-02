//! Galois field arithmetic for codewords and polynomials.

use std;

use collect_slice::CollectSlice;

/// GF(2<sup>6</sup>) field characterized by α<sup>6</sup>+α+1, as described in the P25
/// specification.
#[derive(Copy, Clone, Debug)]
pub struct P25Field;

impl GaloisField for P25Field {
    fn size() -> usize { 63 }
    fn valid_codeword(bits: u8) -> bool { bits >> 6 == 0 }

    fn codeword(pow: usize) -> u8 {
        // Each codeword α^i represents the polynomial x^i mod h(x), where P25 uses h(x) =
        // x^6 + x + 1.
        const CODEWORDS: [u8; 63] = [
            0b000001,
            0b000010,
            0b000100,
            0b001000,
            0b010000,
            0b100000,
            0b000011,
            0b000110,
            0b001100,
            0b011000,
            0b110000,
            0b100011,
            0b000101,
            0b001010,
            0b010100,
            0b101000,
            0b010011,
            0b100110,
            0b001111,
            0b011110,
            0b111100,
            0b111011,
            0b110101,
            0b101001,
            0b010001,
            0b100010,
            0b000111,
            0b001110,
            0b011100,
            0b111000,
            0b110011,
            0b100101,
            0b001001,
            0b010010,
            0b100100,
            0b001011,
            0b010110,
            0b101100,
            0b011011,
            0b110110,
            0b101111,
            0b011101,
            0b111010,
            0b110111,
            0b101101,
            0b011001,
            0b110010,
            0b100111,
            0b001101,
            0b011010,
            0b110100,
            0b101011,
            0b010101,
            0b101010,
            0b010111,
            0b101110,
            0b011111,
            0b111110,
            0b111111,
            0b111101,
            0b111001,
            0b110001,
            0b100001,
        ];

        CODEWORDS[pow]
    }

    fn power(codeword: usize) -> usize {
        const POWERS: [usize; 63] = [
            0,
            1,
            6,
            2,
            12,
            7,
            26,
            3,
            32,
            13,
            35,
            8,
            48,
            27,
            18,
            4,
            24,
            33,
            16,
            14,
            52,
            36,
            54,
            9,
            45,
            49,
            38,
            28,
            41,
            19,
            56,
            5,
            62,
            25,
            11,
            34,
            31,
            17,
            47,
            15,
            23,
            53,
            51,
            37,
            44,
            55,
            40,
            10,
            61,
            46,
            30,
            50,
            22,
            39,
            43,
            29,
            60,
            42,
            21,
            20,
            59,
            57,
            58,
        ];

        POWERS[codeword]
    }
}

/// Codeword in the P25 Galois field.
pub type P25Codeword = Codeword<P25Field>;

/// A GF(2<sup>r</sup>) Galois field.
pub trait GaloisField {
    /// Number of unique codewords in the field: 2<sup>r</sup> - 1.
    fn size() -> usize;
    /// Check if the given bit pattern is a valid codeword in the field.
    fn valid_codeword(bits: u8) -> bool;
    /// Map the given power i to codeword α<sup>i</sup>.
    fn codeword(pow: usize) -> u8;
    /// Map the given codeword a<sup>i</sup> to its power i.
    fn power(codeword: usize) -> usize;

    /// Map the given power i to codeword α<sup>m</sup> ≡ α<sup>i</sup> (modulo the size
    /// of the field.)
    fn codeword_modded(pow: usize) -> u8 {
        Self::codeword(pow % Self::size())
    }
}

/// Codeword in a Galois field.
#[derive(Copy, Clone)]
pub struct Codeword<F: GaloisField> {
    field: std::marker::PhantomData<F>,
    bits: u8,
}

impl<F: GaloisField> Codeword<F> {
    /// Construct a new `Codeword` α<sup>i</sup> from the given bit pattern. Panic if the
    /// pattern is invalid in the field.
    pub fn new(bits: u8) -> Codeword<F> {
        assert!(F::valid_codeword(bits));

        Codeword {
            field: std::marker::PhantomData,
            bits: bits,
        }
    }

    /// Construct a new `Codeword` α<sup>m</sup> ≡ α<sup>i</sup> (modulo the field) for
    /// the given power i.
    pub fn for_power(power: usize) -> Codeword<F> {
        Codeword::new(F::codeword_modded(power))
    }

    /// Retrieve the bit pattern of the codeword.
    pub fn bits(&self) -> u8 { self.bits }

    /// Check if the codeword is zero.
    pub fn zero(&self) -> bool { self.bits == 0 }

    /// Retrieve the power i of the current codeword α<sup>i</sup>. Return `Some(i)` if
    /// the power is defined and `None` if the codeword is zero.
    pub fn power(&self) -> Option<usize> {
        if self.zero() {
            None
        } else {
            // Convert to zero-based index.
            Some(F::power(self.bits as usize - 1))
        }
    }

    /// Find 1/α<sup>i</sup> for the current codeword α<sup>i</sup>. Panic if the codeword
    /// is zero.
    pub fn invert(self) -> Codeword<F> {
        match self.power() {
            Some(p) => Codeword::for_power(F::size() - p),
            None => panic!("invert zero"),
        }
    }

    /// Compute (α<sup>i</sup>)<sup>p</sup> for the current codeword α<sup>i</sup> and
    /// given power p.
    pub fn pow(&self, pow: usize) -> Codeword<F> {
        match self.power() {
            Some(p) => Codeword::for_power(p * pow),
            None => Codeword::default(),
        }
    }
}

impl<F: GaloisField> Default for Codeword<F> {
    /// Construct the additive identity codeword α<sup>0</sup> = 1.
    fn default() -> Self {
        Codeword::new(0)
    }
}

/// Add codewords using Galois addition.
impl<F: GaloisField> std::ops::Add for Codeword<F> {
    type Output = Codeword<F>;

    fn add(self, rhs: Codeword<F>) -> Self::Output {
        Codeword::new(self.bits ^ rhs.bits)
    }
}

/// "Subtract" codewords, which is equivalent to addition.
impl<F: GaloisField> std::ops::Sub for Codeword<F> {
    type Output = Codeword<F>;

    fn sub(self, rhs: Codeword<F>) -> Self::Output {
        self + rhs
    }
}

/// Mutiply codewords using Galois multiplication.
impl<F: GaloisField> std::ops::Mul for Codeword<F> {
    type Output = Codeword<F>;

    fn mul(self, rhs: Codeword<F>) -> Self::Output {
        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Codeword::for_power(p + q),
            _ => Codeword::default(),
        }
    }
}

/// Divide codewords using Galois division. Panic if the divisor is zero.
impl<F: GaloisField> std::ops::Div for Codeword<F> {
    type Output = Codeword<F>;

    fn div(self, rhs: Codeword<F>) -> Self::Output {
        match (self.power(), rhs.power()) {
            // Ensure non-negative power.
            (Some(p), Some(q)) => Codeword::for_power(F::size() + p - q),
            (None, Some(_)) => Codeword::default(),
            (_, None) => panic!("divide by zero"),
        }
    }
}

/// Check equality of two codewords.
impl<F: GaloisField> std::cmp::PartialEq for Codeword<F> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<F: GaloisField> std::cmp::Eq for Codeword<F> {}

/// Check equality of the codeword's bit pattern with raw bits.
impl<F: GaloisField> std::cmp::PartialEq<u8> for Codeword<F> {
    fn eq(&self, other: &u8) -> bool {
        self.bits == *other
    }
}

impl<F: GaloisField> std::fmt::Debug for Codeword<F> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.power() {
            Some(p) => write!(fmt, "Codeword::for_power({})", p),
            None => write!(fmt, "Codeword::default()"),
        }
    }
}

/// Coefficient storage for a bounded-degree Galois polynomial of a particular code.
pub trait PolynomialCoefs: Default + Copy + Clone +
    std::ops::Deref<Target = [P25Codeword]> + std::ops::DerefMut
{
    /// The minimum Hamming distance, d, in (n,k,d).
    fn distance() -> usize;

    /// Maximum number of correctable errors: t.
    fn errors() -> usize {
        // Since d is odd, d = 2t+1 ⇒ t = (d-1)/2 = floor(d / 2)
        Self::distance() / 2
    }

    /// Number of syndromes: 2t.
    fn syndromes() -> usize { 2 * Self::errors() }

    /// Verify the implementer is well-formed.
    fn validate(&self) {
        // Distance must be odd.
        assert!(Self::distance() % 2 == 1);
        // Storage must at least be able to hold a full syndrome polynomial.
        assert!(self.len() >= Self::syndromes());
    }
}

/// Create a coefficient storage buffer for the code of given distance. In the first form,
/// the polynomial is large enough to store the Berlekamp-Massey decoding polynomials. In
/// the second form, the polynomial has the given size.
macro_rules! impl_polynomial_coefs {
    ($name:ident, $dist:expr) => {
        impl_polynomial_coefs!($name, $dist, $dist + 1);
    };
    ($name:ident, $dist:expr, $len:expr) => {
        #[derive(Copy)]
        struct $name([P25Codeword; $len]);

        impl PolynomialCoefs for $name {
            fn distance() -> usize { $dist }
        }

        impl Default for $name {
            fn default() -> Self {
                $name([P25Codeword::default(); $len])
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                let mut coefs = [P25Codeword::default(); $len];
                coefs.copy_from_slice(&self.0[..]);
                $name(coefs)
            }
        }

        impl std::ops::Deref for $name {
            type Target = [P25Codeword];
            fn deref(&self) -> &Self::Target { &self.0[..] }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
        }
    };
}

/// Polynomial with P25's GF(2<sup>6</sup>) codewords as coefficients.
#[derive(Copy, Clone)]
pub struct Polynomial<P: PolynomialCoefs> {
    /// Coefficients of the polynomial. The maximum degree span in the algorithm is [0,
    /// 2t+1], or 2t+2 coefficients.
    coefs: P,
    /// Index into `coefs` of the degree-0 coefficient. Coefficients with a lesser index
    /// will be zero.
    start: usize,
}

impl<P: PolynomialCoefs> Polynomial<P> {
    /// Construct a new `Polynomial` from the given coefficients c<sub>0</sub>, ...,
    /// c<sub>k</sub>.
    ///
    /// The resulting polynomial has the form p(x) = c<sub>0</sub> + c<sub>1</sub>x + ···
    /// + c<sub>k</sub>x<sup>k</sup>.
    pub fn new<T: Iterator<Item = P25Codeword>>(mut init: T) -> Self {
        // Start with all zero coefficients and add in the given ones.
        let mut coefs = P::default();
        init.collect_slice_exhaust(&mut coefs[..]);

        Self::with_coefs(coefs)
    }

    /// Construct a new `Polynomial` with the single term p(x) = x<sup>n</sup>.
    pub fn unit_power(n: usize) -> Self {
        let mut coefs = P::default();
        coefs[n] = Codeword::for_power(0);

        Self::with_coefs(coefs)
    }

    /// Construct a new `Polynomial` with the given polynomials.
    fn with_coefs(coefs: P) -> Self {
        Polynomial {
            coefs: coefs,
            start: 0,
        }
    }

    /// Retrieve the degree-0 coefficient, c<sub>0</sub>.
    pub fn constant(&self) -> P25Codeword {
        self.coefs[self.start]
    }

    /// Compute deg(p(x)), returned as `Some(deg)` if the polynomial is nonzero, or
    /// `None` if p(x) = 0.
    ///
    /// Note this is a O(n) operation.
    pub fn degree(&self) -> Option<usize> {
        for (deg, coef) in self.coefs.iter().enumerate().rev() {
            if !coef.zero() {
                // Any coefficients before `start` aren't part of the polynomial.
                return Some(deg - self.start);
            }
        }

        None
    }

    /// Divide the polynomial by x -- shift all coefficients to a lower degree. Panic if
    /// c<sub>0</sub> ≠ 0.
    ///
    /// This is a O(1) operation.
    pub fn shift(mut self) -> Polynomial<P> {
        assert!(self.constant().zero());

        self.coefs[self.start] = P25Codeword::default();
        self.start += 1;
        self
    }

    /// Retrieve the coefficient at the given absolute index into the storage buffer, or 0
    /// if the index is out of bounds.
    fn get(&self, idx: usize) -> P25Codeword {
        match self.coefs.get(idx) {
            Some(&c) => c,
            None => P25Codeword::default(),
        }
    }

    /// Retrieve the coefficient c<sub>i</sub> associated with the x<sup>i</sup> term.
    ///
    /// If i > deg(p(x)), 0 is returned.
    pub fn coef(&self, i: usize) -> P25Codeword {
        self.get(self.start + i)
    }

    /// Evaluate p(x), substituting in the given x.
    pub fn eval(&self, x: P25Codeword) -> P25Codeword {
        // This uses Horner's method which, unlike the naive method, doesn't require a
        // call to `pow()` at each term.
        self.iter().rev().fold(P25Codeword::default(), |s, &coef| s * x + coef)
    }

    /// Truncate the polynomial so that deg(p(x)) ≤ d, where d is the given degree.
    ///
    /// This is a O(n) operation.
    pub fn truncate(mut self, deg: usize) -> Polynomial<P> {
        for i in (self.start + deg + 1)..self.coefs.len() {
            self.coefs[i] = P25Codeword::default();
        }

        self
    }

    /// Compute the formal derivative p'(x).
    pub fn deriv(mut self) -> Polynomial<P> {
        for i in self.start..self.coefs.len() {
            self.coefs[i] = if (i - self.start) % 2 == 0 {
                self.get(i + 1)
            } else {
                P25Codeword::default()
            };
        }

        self
    }
}

impl<P: PolynomialCoefs> Default for Polynomial<P> {
    /// Construct an empty polynomial, p(x) = 0.
    fn default() -> Self {
        Polynomial::new(std::iter::empty())
    }
}

/// Provides a slice of coefficients starting at the degree-0 term, [c<sub>0</sub>,
/// c<sub>1</sub>, ...].
impl<P: PolynomialCoefs> std::ops::Deref for Polynomial<P> {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.coefs[self.start..] }
}

impl<P: PolynomialCoefs> std::ops::DerefMut for Polynomial<P> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.coefs[self.start..] }
}

/// Add polynomials using Galois addition for coefficients.
impl<P: PolynomialCoefs> std::ops::Add for Polynomial<P> {
    type Output = Polynomial<P>;

    fn add(mut self, rhs: Polynomial<P>) -> Self::Output {
        // Sum the coefficients and reset the degree-0 term back to index 0.
        //
        // Since start >= 0 => start+i >= i, so there's no overwriting.
        for i in 0..self.coefs.len() {
            self.coefs[i] = self.coef(i) + rhs.coef(i);
        }

        self.start = 0;
        self
    }
}

/// Scale polynomial by a codeword.
impl<P: PolynomialCoefs> std::ops::Mul<P25Codeword> for Polynomial<P> {
    type Output = Polynomial<P>;

    fn mul(mut self, rhs: P25Codeword) -> Self::Output {
        for coef in self.coefs.iter_mut() {
            *coef = *coef * rhs;
        }

        self
    }
}

/// Multiply polynomials using Galois multiplication for coefficients.
///
/// Note that resulting terms outside the bounds of the polynomial are silently discarded,
/// effectively computing p(x)q(x) mod x<sup>n+1</sup>, where n is the maximum degree
/// supported by the polynomial.
impl<P: PolynomialCoefs> std::ops::Mul<Polynomial<P>> for Polynomial<P> {
    type Output = Polynomial<P>;

    fn mul(self, rhs: Polynomial<P>) -> Self::Output {
        let mut out = Polynomial::<P>::default();

        for (i, &coef) in self.iter().enumerate() {
            for (j, &mult) in rhs.iter().enumerate() {
                match out.coefs.get_mut(i + j) {
                    Some(c) => *c = *c + coef * mult,
                    None => {},
                }
            }
        }

        out
    }
}

impl<P: PolynomialCoefs> std::fmt::Debug for Polynomial<P> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Polynomial({:?})", &self.coefs[..])
    }
}

#[cfg(test)]
mod test {
    use std;
    use super::*;

    #[derive(Copy, Clone, Default)]
    struct TestCoefs([P25Codeword; 24]);

    impl std::ops::Deref for TestCoefs {
        type Target = [P25Codeword];
        fn deref(&self) -> &Self::Target { &self.0[..] }
    }

    impl std::ops::DerefMut for TestCoefs {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
    }

    impl PolynomialCoefs for TestCoefs {
        fn distance() -> usize { 23 }
    }

    type TestPolynomial = Polynomial<TestCoefs>;

    #[derive(Copy, Clone, Default)]
    struct ShortCoefs([P25Codeword; 5]);

    impl std::ops::Deref for ShortCoefs {
        type Target = [P25Codeword];
        fn deref(&self) -> &Self::Target { &self.0[..] }
    }

    impl std::ops::DerefMut for ShortCoefs {
        fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0[..] }
    }

    impl PolynomialCoefs for ShortCoefs {
        fn distance() -> usize { 3 }
    }

    type ShortPolynomial = Polynomial<ShortCoefs>;

    #[test]
    fn test_coefs() {
        assert_eq!(TestCoefs::errors(), 11);
        assert_eq!(TestCoefs::syndromes(), 22);
    }

    #[test]
    fn test_for_power() {
        assert!(P25Codeword::for_power(0) == 0b000001);
        assert!(P25Codeword::for_power(62) == 0b100001);
        assert!(P25Codeword::for_power(63) == 0b000001);
    }

    #[test]
    fn test_add_sub() {
        assert!((P25Codeword::new(0b100000) + P25Codeword::new(0b010000)) == 0b110000);
        assert!((P25Codeword::new(0b100000) - P25Codeword::new(0b010000)) == 0b110000);
        assert!((P25Codeword::new(0b100001) + P25Codeword::new(0b100001)) == 0b000000);
        assert!((P25Codeword::new(0b100001) - P25Codeword::new(0b100001)) == 0b000000);
        assert!((P25Codeword::new(0b100001) + P25Codeword::new(0b110100)) == 0b010101);
        assert!((P25Codeword::new(0b100001) - P25Codeword::new(0b110100)) == 0b010101);
    }

    #[test]
    fn test_mul() {
        assert!((P25Codeword::new(0b000110) * P25Codeword::new(0b000101)) == 0b011110);
        assert!((P25Codeword::new(0b000000) * P25Codeword::new(0b000101)) == 0b000000);
        assert!((P25Codeword::new(0b000110) * P25Codeword::new(0b000000)) == 0b000000);
        assert!((P25Codeword::new(0b000000) * P25Codeword::new(0b000000)) == 0b000000);
        assert!((P25Codeword::new(0b100001) * P25Codeword::new(0b000001)) == 0b100001);
        assert!((P25Codeword::new(0b100001) * P25Codeword::new(0b000010)) == 0b000001);
        assert!((P25Codeword::new(0b110011) * P25Codeword::new(0b110011)) == 0b111001);
        assert!((P25Codeword::new(0b101111) * P25Codeword::new(0b101111)) == 0b100110);
    }


    #[test]
    fn test_div() {
        assert!((P25Codeword::new(0b001000) / P25Codeword::new(0b000101)) == 0b010111);
        assert!((P25Codeword::new(0b000000) / P25Codeword::new(0b101000)) == 0b000000);
        assert!((P25Codeword::new(0b011110) / P25Codeword::new(0b000001)) == 0b011110);
        assert!((P25Codeword::new(0b011110) / P25Codeword::new(0b011110)) == 0b000001);
    }

    #[test]
    fn test_cmp() {
        assert!(P25Codeword::new(0b000000) == P25Codeword::new(0b000000));
    }

    #[test]
    fn test_pow() {
        assert_eq!(P25Codeword::for_power(0).pow(10).power().unwrap(), 0);
        assert_eq!(P25Codeword::for_power(1).pow(10).power().unwrap(), 10);
        assert_eq!(P25Codeword::for_power(62).pow(10).power().unwrap(), 53);
        assert!(P25Codeword::default().pow(20).power().is_none());
    }

    #[test]
    fn test_eval() {
        let p = TestPolynomial::new((0..3).map(|_| {
            P25Codeword::for_power(0)
        }));
        assert!(p.eval(P25Codeword::for_power(1)) == 0b000111);

        let p = TestPolynomial::new((0..2).map(|_| {
            P25Codeword::for_power(0)
        }));
        assert_eq!(p.eval(P25Codeword::for_power(1)), 0b000011);

        let p = TestPolynomial::new([
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned());
        assert_eq!(p.eval(P25Codeword::for_power(3)), 0b011000);

        let p = TestPolynomial::new([
            P25Codeword::default(),
            P25Codeword::for_power(0),
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned());
        assert_eq!(p.eval(P25Codeword::for_power(3)), 0b010000);

        let p = TestPolynomial::new([
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(5),
        ].iter().cloned());
        assert_eq!(p.eval(P25Codeword::for_power(4)), 0b100100);

        let p = TestPolynomial::new([
            P25Codeword::for_power(12),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::default(),
            P25Codeword::for_power(5),
        ].iter().cloned());
        assert_eq!(p.eval(P25Codeword::for_power(4)), 0b100001);

        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
        ].iter().cloned());
        assert!(p.eval(P25Codeword::for_power(0)).zero());
    }

    #[test]
    fn test_truncate() {
        let p = TestPolynomial::new((0..5).map(|_| {
            P25Codeword::for_power(0)
        }));

        assert_eq!(p.degree().unwrap(), 4);
        assert_eq!(p.coefs[4].power().unwrap(), 0);
        assert!(p.coefs[5].power().is_none());

        let p = p.truncate(2);
        assert_eq!(p.degree().unwrap(), 2);
        assert_eq!(p.coefs[2].power().unwrap(), 0);
        assert!(p.coefs[3].power().is_none());
    }

    #[test]
    fn test_polynomial() {
        let p = TestPolynomial::new((0..23).map(|i| {
            P25Codeword::for_power(i)
        }));

        assert!(p.degree().unwrap() == 22);
        assert!(p.constant() == P25Codeword::for_power(0));

        let p = TestPolynomial::new((1..23).map(|i| {
            P25Codeword::for_power(i)
        }));
        assert!(p.degree().unwrap() == 21);
        assert!(p.constant() == P25Codeword::for_power(1));

        let q = p.clone() * P25Codeword::for_power(0);
        assert!(q.degree().unwrap() == 21);
        assert!(q.constant() == P25Codeword::for_power(1));

        let q = p.clone() * P25Codeword::for_power(2);
        assert!(q.degree().unwrap() == 21);
        assert!(q.constant() == P25Codeword::for_power(3));

        let q = p.clone() + p.clone();
        assert!(q.constant().zero());

        for coef in q.iter() {
            assert!(coef.zero());
        }

        let p = TestPolynomial::new((4..27).map(|i| {
            P25Codeword::for_power(i)
        }));

        let q = TestPolynomial::new((4..26).map(|i| {
            P25Codeword::for_power(i)
        }));

        let r = p + q;

        assert!(r.coefs[0].zero());
        assert!(r.coefs[1].zero());
        assert!(r.coefs[2].zero());
        assert!(r.coefs[3].zero());
        assert!(r.coefs[4].zero());
        assert!(!r.coefs[22].zero());

        let p = TestPolynomial::new((0..2).map(|_| {
            P25Codeword::for_power(0)
        }));

        let q = TestPolynomial::new((0..4).map(|_| {
            P25Codeword::for_power(1)
        }));

        let r = p + q;

        assert!(r.coef(0) == P25Codeword::for_power(6));
    }

    #[test]
    fn test_poly_mul() {
        let p = TestPolynomial::new((0..2).map(|_| {
            P25Codeword::for_power(0)
        }));

        let q = p.clone();
        let r = p * q;

        assert_eq!(r.coef(0).power().unwrap(), 0);
        assert!(r.coef(1).power().is_none());
        assert_eq!(r.coef(2).power().unwrap(), 0);

        let p = TestPolynomial::new((0..3).map(|p| {
            P25Codeword::for_power(p)
        }));
        let q = TestPolynomial::new([
            P25Codeword::default(),
            P25Codeword::for_power(0),
        ].iter().cloned());
        let r = p * q;

        assert!(r.coef(0).power().is_none());
        assert_eq!(r.coef(1).power().unwrap(), 0);
        assert_eq!(r.coef(2).power().unwrap(), 1);
        assert_eq!(r.coef(3).power().unwrap(), 2);
    }

    #[test]
    fn test_deriv() {
        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned());

        let q = p.deriv();

        assert!(q.coefs[0] == P25Codeword::for_power(3));
        assert!(q.coefs[1] == P25Codeword::default());
        assert!(q.coefs[2] == P25Codeword::default());

        let p = TestPolynomial::new([
            P25Codeword::default(),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned());

        let q = p.shift().deriv();

        assert!(q.coef(0) == P25Codeword::for_power(3));
        assert!(q.coef(1) == P25Codeword::default());
        assert!(q.coef(2) == P25Codeword::default());

        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned()).deriv();

        assert!(p.coef(0) == P25Codeword::for_power(5));
        assert!(p.coef(1) == P25Codeword::default());
        assert!(p.coef(2) == P25Codeword::for_power(58));
        assert!(p.coef(3) == P25Codeword::default());
        assert!(p.coef(4) == P25Codeword::default());

        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
        ].into_iter().cloned()).deriv();

        assert!(p.coef(0) == P25Codeword::for_power(5));
        assert!(p.coef(1) == P25Codeword::default());
        assert!(p.coef(2) == P25Codeword::for_power(58));
        assert!(p.coef(3) == P25Codeword::default());
        assert!(p.coef(4) == P25Codeword::default());
        assert!(p.coef(5) == P25Codeword::default());

        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
            P25Codeword::for_power(15),
        ].into_iter().cloned()).deriv();

        assert!(p.coef(0) == P25Codeword::for_power(5));
        assert!(p.coef(1) == P25Codeword::default());
        assert!(p.coef(2) == P25Codeword::for_power(58));
        assert!(p.coef(3) == P25Codeword::default());
        assert!(p.coef(4) == P25Codeword::for_power(15));
        assert!(p.coef(5) == P25Codeword::default());

        let p = ShortPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
        ].into_iter().cloned()).deriv();

        assert!(p.coef(0) == P25Codeword::for_power(5));
        assert!(p.coef(1) == P25Codeword::default());
        assert!(p.coef(2) == P25Codeword::for_power(58));
        assert!(p.coef(3) == P25Codeword::default());
        assert!(p.coef(4) == P25Codeword::default());

        let p = TestPolynomial::new([
            P25Codeword::for_power(0),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
            P25Codeword::for_power(15),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
            P25Codeword::for_power(15),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
            P25Codeword::for_power(15),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
            P25Codeword::for_power(43),
            P25Codeword::for_power(15),
            P25Codeword::for_power(5),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned()).deriv();

        assert!(p.coef(0) == P25Codeword::for_power(5));
        assert!(p.coef(1) == P25Codeword::default());
        assert!(p.coef(2) == P25Codeword::for_power(58));
        assert!(p.coef(3) == P25Codeword::default());
        assert!(p.coef(4) == P25Codeword::for_power(15));
        assert!(p.coef(5) == P25Codeword::default());
        assert!(p.coef(6) == P25Codeword::for_power(3));
        assert!(p.coef(7) == P25Codeword::default());
        assert!(p.coef(8) == P25Codeword::for_power(43));
        assert!(p.coef(9) == P25Codeword::default());
        assert!(p.coef(10) == P25Codeword::for_power(5));
        assert!(p.coef(11) == P25Codeword::default());
        assert!(p.coef(12) == P25Codeword::for_power(58));
        assert!(p.coef(13) == P25Codeword::default());
        assert!(p.coef(14) == P25Codeword::for_power(15));
        assert!(p.coef(15) == P25Codeword::default());
        assert!(p.coef(16) == P25Codeword::for_power(3));
        assert!(p.coef(17) == P25Codeword::default());
        assert!(p.coef(18) == P25Codeword::for_power(43));
        assert!(p.coef(19) == P25Codeword::default());
        assert!(p.coef(20) == P25Codeword::for_power(5));
        assert!(p.coef(21) == P25Codeword::default());
        assert!(p.coef(22) == P25Codeword::for_power(58));
        assert!(p.coef(23) == P25Codeword::default());
        assert!(p.coef(24) == P25Codeword::default());
    }

    #[test]
    fn test_unit_power() {
        let p = TestPolynomial::unit_power(0);
        assert_eq!(p[0], Codeword::for_power(0));
        assert_eq!(p.degree().unwrap(), 0);

        let p = TestPolynomial::unit_power(2);
        assert_eq!(p[0], Codeword::default());
        assert_eq!(p[1], Codeword::default());
        assert_eq!(p[2], Codeword::for_power(0));
        assert_eq!(p.degree().unwrap(), 2);

        let p = TestPolynomial::unit_power(10);
        assert_eq!(p[0], Codeword::default());
        assert_eq!(p[1], Codeword::default());
        assert_eq!(p[2], Codeword::default());
        assert_eq!(p[3], Codeword::default());
        assert_eq!(p[4], Codeword::default());
        assert_eq!(p[5], Codeword::default());
        assert_eq!(p[6], Codeword::default());
        assert_eq!(p[7], Codeword::default());
        assert_eq!(p[8], Codeword::default());
        assert_eq!(p[9], Codeword::default());
        assert_eq!(p[10], Codeword::for_power(0));
        assert_eq!(p.degree().unwrap(), 10);
    }
}
