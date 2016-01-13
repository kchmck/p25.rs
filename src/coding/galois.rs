use std;

use collect_slice::CollectSlice;

/// GF(2^6) field characterized by α^6+α+1.as described in P25 specification.
#[derive(Copy, Clone, Debug)]
pub struct P25Field;

impl GaloisField for P25Field {
    fn size() -> usize { 63 }

    fn codeword(pow: usize) -> u8 {
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

pub type P25Codeword = Codeword<P25Field>;

pub trait GaloisField {
    /// Number of unique codewords in the field.
    fn size() -> usize;
    /// Maps the given i in α^i to its codeword.
    fn codeword(pow: usize) -> u8;
    /// Maps the given codeword to i in α^i.
    fn power(codeword: usize) -> usize;

    /// Maps the given i in α^i, modulo the size of the field, to its codeword.
    fn codeword_modded(pow: usize) -> u8 {
        Self::codeword(pow % Self::size())
    }
}

#[derive(Copy, Clone)]
/// Codeword in the associated field.
pub struct Codeword<F: GaloisField> {
    field: std::marker::PhantomData<F>,
    bits: u8,
}

impl<F: GaloisField> Codeword<F> {
    /// Construct a new `Codeword` with the given (valid) codeword in the field.
    pub fn new(codeword: u8) -> Codeword<F> {
        Codeword {
            field: std::marker::PhantomData,
            bits: codeword,
        }
    }

    /// Get the wrapped codeword bits.
    pub fn bits(&self) -> u8 { self.bits }

    /// Check if the codeword is zero.
    pub fn zero(&self) -> bool { self.bits == 0 }

    /// Return `Some(i)` if the codeword is equal to α^i and `None` if it's equal to zero.
    pub fn power(&self) -> Option<usize> {
        if self.zero() {
            None
        } else {
            // Convert to zero-based index.
            Some(F::power(self.bits as usize - 1))
        }
    }

    /// Return the codeword for the given power, which is cyclic in the field.
    pub fn for_power(power: usize) -> Codeword<F> {
        Codeword::new(F::codeword_modded(power))
    }

    /// Find 1/a^i for the codeword equal to a^i. Panic if the codeword is zero.
    pub fn invert(self) -> Codeword<F> {
        match self.power() {
            Some(p) => Codeword::for_power(F::size() - p),
            None => panic!("divide by zero"),
        }
    }

    /// Raise codeword to the given power.
    pub fn pow(&self, pow: usize) -> Codeword<F> {
        match self.power() {
            Some(p) => Codeword::for_power(p * pow),
            None => Codeword::default(),
        }
    }
}

impl<F: GaloisField> Default for Codeword<F> {
    /// Get the additive identity codeword.
    fn default() -> Self {
        Codeword::new(0)
    }
}

impl<F: GaloisField> std::ops::Mul for Codeword<F> {
    type Output = Codeword<F>;

    fn mul(self, rhs: Codeword<F>) -> Self::Output {
        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Codeword::for_power(p + q),
            _ => Codeword::default(),
        }
    }
}

impl<F: GaloisField> std::ops::Div for Codeword<F> {
    type Output = Codeword<F>;

    fn div(self, rhs: Codeword<F>) -> Self::Output {
        match (self.power(), rhs.power()) {
            // max(q) = 62 => 63-max(power) > 0
            (Some(p), Some(q)) => Codeword::for_power(p + F::size() - q),
            (None, Some(_)) => Codeword::default(),
            (_, None) => panic!("divide by zero"),
        }
    }
}

impl<F: GaloisField> std::ops::Add for Codeword<F> {
    type Output = Codeword<F>;

    fn add(self, rhs: Codeword<F>) -> Self::Output {
        Codeword::new(self.bits ^ rhs.bits)
    }
}

impl<F: GaloisField> std::ops::Sub for Codeword<F> {
    type Output = Codeword<F>;

    fn sub(self, rhs: Codeword<F>) -> Self::Output {
        self + rhs
    }
}

impl<F: GaloisField> std::cmp::PartialEq for Codeword<F> {
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl<F: GaloisField> std::cmp::Eq for Codeword<F> {}

impl<F: GaloisField> std::cmp::PartialEq<u8> for Codeword<F> {
    fn eq(&self, other: &u8) -> bool {
        self.bits == *other
    }
}

/// Wraps a static codeword array.
pub trait PolynomialCoefs: Default + Copy + Clone + std::ops::Deref<Target = [P25Codeword]> +
    std::ops::DerefMut
{
    /// The `d` Hamming distance in (n,k,d).
    fn distance() -> usize;

    /// Maximum number of correctable errors: d=2t+1 => t = (d-1)/2 = floor(d/2)
    fn errors() -> usize { Self::distance() / 2 }
    /// Number of syndromes needed, degree of syndrome polynomial.
    fn syndromes() -> usize { Self::errors() * 2 }

    /// Verify the implementer is well-formed.
    fn validate(&self) {
        assert!(Self::distance() % 2 == 1);
    }
}

/// A syndrome polynomial with GF(2^6) codewords as coefficients.
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
    /// Construct a new `Polynomial` from the given coefficients, so p(x) = coefs[0] +
    /// coefs[1]*x + ... + coefs[n]*x^n.
    pub fn new<T: Iterator<Item = P25Codeword>>(mut init: T) -> Polynomial<P> {
        // Start with all zero coefficients and add in the given ones.
        let mut coefs = P::default();
        init.collect_slice(&mut coefs[..]);

        Polynomial {
            coefs: coefs,
            start: 0,
        }
    }

    /// Get the degree-0 coefficient.
    pub fn constant(&self) -> P25Codeword {
        self.coefs[self.start]
    }

    /// Return `Some(deg)`, where `deg` is the highest degree term in the polynomial, if
    /// the polynomial is nonzero and `None` if it's zero.
    pub fn degree(&self) -> Option<usize> {
        for (deg, coef) in self.coefs.iter().enumerate().rev() {
            if !coef.zero() {
                // Any coefficients before `start` aren't part of the polynomial.
                return Some(deg - self.start);
            }
        }

        None
    }

    /// Divide the polynomial by x -- shift all coefficients to a lower degree -- and
    /// replace the shifted coefficient with the zero codeword. There must be no constant
    /// term.
    pub fn shift(mut self) -> Polynomial<P> {
        self.coefs[self.start] = P25Codeword::default();
        self.start += 1;
        self
    }

    /// Get the coefficient of the given absolute degree if it exists in the polynomial
    /// or the zero codeword if it doesn't.
    fn get(&self, idx: usize) -> P25Codeword {
        match self.coefs.get(idx) {
            Some(&c) => c,
            None => P25Codeword::default(),
        }
    }

    /// Get the coefficient of the given degree or the zero codeword if the degree doesn't
    /// exist in the polynomial.
    pub fn coef(&self, deg: usize) -> P25Codeword {
        self.get(self.start + deg)
    }

    /// Evaluate the polynomial, substituting in `x`.
    pub fn eval(&self, x: P25Codeword) -> P25Codeword {
        self.iter().enumerate().fold(P25Codeword::default(), |s, (pow, coef)| {
            s + *coef * x.pow(pow)
        })
    }

    /// Truncate the polynomial to have no terms greater than the given degree.
    pub fn truncate(mut self, deg: usize) -> Polynomial<P> {
        for i in (self.start + deg + 1)..self.coefs.len() {
            self.coefs[i] = P25Codeword::default();
        }

        self
    }

    /// Take the derivative of the polynomial.
    pub fn deriv(mut self) -> Polynomial<P> {
        for i in self.start..self.coefs.len() {
            if (i - self.start) % 2 == 0 {
                if let Some(&coef) = self.coefs.get(i + 1) {
                    self.coefs[i] = coef
                }
            } else {
                self.coefs[i] = P25Codeword::default();
            }
        }

        self
    }
}

impl<P: PolynomialCoefs> Default for Polynomial<P> {
    fn default() -> Self {
        Polynomial::new(std::iter::empty())
    }
}

impl<P: PolynomialCoefs> std::ops::Deref for Polynomial<P> {
    type Target = [P25Codeword];
    fn deref(&self) -> &Self::Target { &self.coefs[self.start..] }
}

impl<P: PolynomialCoefs> std::ops::DerefMut for Polynomial<P> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.coefs[self.start..] }
}

impl<P: PolynomialCoefs> std::ops::Add for Polynomial<P> {
    type Output = Polynomial<P>;

    fn add(mut self, rhs: Polynomial<P>) -> Self::Output {
        // Sum the coefficients and reset the degree-0 term back to index 0. Since start >
        // 0 => start+i >= i, so there's no overwriting.
        for i in 0..self.coefs.len() {
            self.coefs[i] = self.coef(i) + rhs.coef(i);
        }

        self.start = 0;
        self
    }
}

impl<P: PolynomialCoefs> std::ops::Mul<P25Codeword> for Polynomial<P> {
    type Output = Polynomial<P>;

    fn mul(mut self, rhs: P25Codeword) -> Self::Output {
        for coef in self.coefs.iter_mut() {
            *coef = *coef * rhs;
        }

        self
    }
}

impl<P: PolynomialCoefs> std::ops::Mul<Polynomial<P>> for Polynomial<P> {
    type Output = Polynomial<P>;

    fn mul(self, rhs: Polynomial<P>) -> Self::Output {
        let mut out = Polynomial::<P>::default();

        for (i, coef) in self.iter().enumerate() {
            for (j, mult) in rhs.iter().enumerate() {
                match out.coefs.get_mut(i + j) {
                    Some(c) => *c = *c + *coef * *mult,
                    None => {},
                }
            }
        }

        out
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

        let p = p.shift();
        assert!(p.eval(P25Codeword::for_power(1)) == 0b000011);
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

        let p = p.shift();
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

        let q = TestPolynomial::new((3..26).map(|i| {
            P25Codeword::for_power(i)
        }));

        let r = p + q.shift();

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
    fn test_prime() {
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
            P25Codeword::for_power(0),
            P25Codeword::for_power(0),
            P25Codeword::for_power(3),
            P25Codeword::for_power(58),
        ].into_iter().cloned());

        let q = p.shift().deriv();

        assert!(q.coef(0) == P25Codeword::for_power(3));
        assert!(q.coef(1) == P25Codeword::default());
        assert!(q.coef(2) == P25Codeword::default());
    }
}
