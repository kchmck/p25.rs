use std;

use self::BCHError::*;

pub fn encode(word: u16) -> u64 {
    GEN.iter().fold(0, |accum, row| {
        accum << 1 | ((word & row).count_ones() % 2) as u64
    })
}

pub fn decode(word: u64) -> Result<(u16, usize), BCHError> {
    let poly = BCHDecoder::new(Syndromes::new(word >> 1)).decode();

    let errors = match poly.degree() {
        Some(deg) => deg,
        None => panic!("invalid polynomial"),
    };

    // Even if there are more errors, the BM algorithm produces a polynomial with degree
    // no greater than ERRORS.
    assert!(errors <= ERRORS);

    let locs = ErrorLocations::new(poly.coefs().iter().cloned());

    let (word, count) = locs.take(errors).fold((word, 0), |(word, s), loc| {
        (word ^ 1 << loc, s + 1)
    });

    let data = (word >> 48) as u16;

    if data & 1 ^ data >> 1 & 1 != (word & 1) as u16 {
        return Err(ParityError);
    }

    // "If the Chien Search fails to find v roots of a error locator polynomial of degree
    // v, then the error pattern is an uncorrectable error pattern" -- Lecture 17:
    // Berlekamp-Massey Algorithm for Binary BCH Codes
    if count == errors {
        Ok((data, errors))
    } else {
        Err(UnrecoverableError)
    }
}

const WORD_SIZE: usize = 63;
const DISTANCE: usize = 23;
// 2t+1 = 23 => t = 11
const ERRORS: usize = 11;
const SYNDROMES: usize = 2 * ERRORS;

// Maps α^i to codewords.
const CODEWORDS: &'static [u8] = &[
    0b100000,
    0b010000,
    0b001000,
    0b000100,
    0b000010,
    0b000001,
    0b110000,
    0b011000,
    0b001100,
    0b000110,
    0b000011,
    0b110001,
    0b101000,
    0b010100,
    0b001010,
    0b000101,
    0b110010,
    0b011001,
    0b111100,
    0b011110,
    0b001111,
    0b110111,
    0b101011,
    0b100101,
    0b100010,
    0b010001,
    0b111000,
    0b011100,
    0b001110,
    0b000111,
    0b110011,
    0b101001,
    0b100100,
    0b010010,
    0b001001,
    0b110100,
    0b011010,
    0b001101,
    0b110110,
    0b011011,
    0b111101,
    0b101110,
    0b010111,
    0b111011,
    0b101101,
    0b100110,
    0b010011,
    0b111001,
    0b101100,
    0b010110,
    0b001011,
    0b110101,
    0b101010,
    0b010101,
    0b111010,
    0b011101,
    0b111110,
    0b011111,
    0b111111,
    0b101111,
    0b100111,
    0b100011,
    0b100001
];

// Maps codewords to α^i.
const POWERS: &'static [usize] = &[
    5,
    4,
    10,
    3,
    15,
    9,
    29,
    2,
    34,
    14,
    50,
    8,
    37,
    28,
    20,
    1,
    25,
    33,
    46,
    13,
    53,
    49,
    42,
    7,
    17,
    36,
    39,
    27,
    55,
    19,
    57,
    0,
    62,
    24,
    61,
    32,
    23,
    45,
    60,
    12,
    31,
    52,
    22,
    48,
    44,
    41,
    59,
    6,
    11,
    16,
    30,
    35,
    51,
    38,
    21,
    26,
    47,
    54,
    43,
    18,
    40,
    56,
    58,
];

const GEN: &'static [u16] = &[
    0b1000000000000000,
    0b0100000000000000,
    0b0010000000000000,
    0b0001000000000000,
    0b0000100000000000,
    0b0000010000000000,
    0b0000001000000000,
    0b0000000100000000,
    0b0000000010000000,
    0b0000000001000000,
    0b0000000000100000,
    0b0000000000010000,
    0b0000000000001000,
    0b0000000000000100,
    0b0000000000000010,
    0b0000000000000001,
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

pub enum BCHError {
    UnrecoverableError,
    ParityError,
}

struct Syndromes {
    pow: std::ops::Range<usize>,
    word: u64,
}

impl Syndromes {
    pub fn new(word: u64) -> Syndromes {
        Syndromes {
            pow: 1..DISTANCE,
            word: word,
        }
    }
}

impl Iterator for Syndromes {
    type Item = Codeword;

    fn next(&mut self) -> Option<Self::Item> {
        match self.pow.next() {
            Some(pow) => Some((0..WORD_SIZE).fold(Codeword::default(), |s, b| {
                if self.word >> b & 1 == 0 {
                    s
                } else {
                    s + Codeword::for_power(b * pow)
                }
            })),
            None => None,
        }
    }
}

#[derive(Copy, Clone)]
struct Codeword(u8);

impl Codeword {
    pub fn new(codeword: u8) -> Codeword {
        Codeword(codeword)
    }

    pub fn zero(&self) -> bool { self.0 == 0 }

    pub fn power(&self) -> Option<usize> {
        if self.zero() {
            None
        } else {
            Some(POWERS[self.0 as usize - 1])
        }
    }

    pub fn for_power(power: usize) -> Codeword {
        Codeword::new(CODEWORDS[power % POWERS.len()])
    }

    pub fn invert(self) -> Codeword {
        match self.power() {
            Some(p) => Codeword::for_power(POWERS.len() - p),
            None => panic!("divide by zero"),
        }
    }
}

impl Default for Codeword {
    fn default() -> Self {
        Codeword::new(0)
    }
}

impl std::ops::Mul for Codeword {
    type Output = Codeword;

    fn mul(self, rhs: Codeword) -> Self::Output {
        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Codeword::for_power(p + q),
            _ => Codeword::default(),
        }
    }
}

impl std::ops::Div for Codeword {
    type Output = Codeword;

    fn div(self, rhs: Codeword) -> Self::Output {
        match (self.power(), rhs.power()) {
            // min(power) = -62 => 63+min(power) > 0
            (Some(p), Some(q)) => Codeword::for_power(p + POWERS.len() - q),
            (None, Some(_)) => Codeword::default(),
            (_, None) => panic!("divide by zero"),
        }
    }
}

impl std::ops::Add for Codeword {
    type Output = Codeword;

    fn add(self, rhs: Codeword) -> Self::Output {
        Codeword::new(self.0 ^ rhs.0)
    }
}

impl std::ops::Sub for Codeword {
    type Output = Codeword;

    fn sub(self, rhs: Codeword) -> Self::Output {
        self + rhs
    }
}

impl std::cmp::PartialEq for Codeword {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::cmp::Eq for Codeword {}

impl std::cmp::PartialOrd for Codeword {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::*;

        match (self.power(), rhs.power()) {
            (Some(p), Some(q)) => Some(p.cmp(&q)),
            (Some(_), None) => Some(Greater),
            (None, Some(_)) => Some(Less),
            (None, None) => Some(Equal),
        }
    }
}

impl std::cmp::Ord for Codeword {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

#[derive(Copy, Clone)]
struct Polynomial {
    /// Coefficients of the polynomial.
    coefs: [Codeword; SYNDROMES + 2],
    /// Index into `coefs` of the degree-0 coefficient.
    start: usize,
}

impl Polynomial {
    pub fn new<T: Iterator<Item = Codeword>>(coefs: T) -> Polynomial {
        let mut poly = [Codeword::default(); SYNDROMES + 2];

        for (cur, coef) in poly.iter_mut().zip(coefs) {
            *cur = *cur + coef;
        }

        Polynomial {
            coefs: poly,
            start: 0,
        }
    }

    pub fn constant(&self) -> Codeword {
        self.coefs[self.start]
    }

    pub fn coefs(&self) -> &[Codeword] {
        &self.coefs[self.start..]
    }

    pub fn degree(&self) -> Option<usize> {
        for (deg, coef) in self.coefs.iter().enumerate().rev() {
            if !coef.zero() {
                return Some(deg - self.start);
            }
        }

        None
    }

    pub fn shift(mut self) -> Polynomial {
        self.coefs[self.start] = Codeword::default();
        self.start += 1;
        self
    }

    fn get(&self, idx: usize) -> Codeword {
        match self.coefs.get(idx) {
            Some(&c) => c,
            None => Codeword::default(),
        }
    }

    pub fn coef(&self, deg: usize) -> Codeword {
        self.get(self.start + deg)
    }
}

impl std::ops::Add for Polynomial {
    type Output = Polynomial;

    fn add(mut self, rhs: Polynomial) -> Self::Output {
        for i in 0..self.coefs.len() {
            self.coefs[i] = self.coef(i) + rhs.coef(i);
        }

        self.start = 0;
        self
    }
}

impl std::ops::Mul<Codeword> for Polynomial {
    type Output = Polynomial;

    fn mul(mut self, rhs: Codeword) -> Self::Output {
        for coef in self.coefs.iter_mut() {
            *coef = *coef * rhs;
        }

        self
    }
}

struct BCHDecoder {
    p_cur: Polynomial,
    p_saved: Polynomial,
    q_cur: Polynomial,
    q_saved: Polynomial,
    deg_saved: usize,
    deg_cur: usize,
}

impl BCHDecoder {
    pub fn new<T: Iterator<Item = Codeword>>(syndromes: T) -> BCHDecoder {
        let q = Polynomial::new(std::iter::once(Codeword::for_power(0))
                                    .chain(syndromes.into_iter()));
        let p = Polynomial::new((0..SYNDROMES+1).map(|_| Codeword::default())
                                    .chain(std::iter::once(Codeword::for_power(0))));

        BCHDecoder {
            q_saved: q,
            q_cur: q.shift(),
            p_saved: p,
            p_cur: p.shift(),
            deg_saved: 0,
            deg_cur: 1,
        }
    }

    pub fn decode(mut self) -> Polynomial {
        for _ in 0..SYNDROMES {
            self.step();
        }

        self.p_cur
    }

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

    fn reduce(&mut self) -> (bool, Polynomial, Polynomial, usize) {
        (
            false,
            self.q_cur.shift(),
            self.p_cur.shift(),
            2 + self.deg_cur,
        )
    }

    fn transform(&mut self) -> (bool, Polynomial, Polynomial, usize) {
        let mult = self.q_cur.constant() / self.q_saved.constant();

        (
            self.deg_cur >= self.deg_saved,
            (self.q_cur + self.q_saved * mult).shift(),
            (self.p_cur + self.p_saved * mult).shift(),
            2 + std::cmp::min(self.deg_cur, self.deg_saved),
        )
   }
}

struct ErrorLocations {
    terms: [Codeword; ERRORS + 1],
    pow: std::ops::Range<usize>,
}

impl ErrorLocations {
    // Λ(x) = coefs[0] + coefs[1]*x + coefs[2]*x^2 + ...
    pub fn new<T: Iterator<Item = Codeword>>(coefs: T) -> ErrorLocations {
        let mut poly = [Codeword::default(); ERRORS + 1];

        for (pow, (cur, coef)) in poly.iter_mut().zip(coefs).enumerate() {
            *cur = *cur + coef / Codeword::for_power(pow)
        }

        ErrorLocations {
            terms: poly,
            pow: 0..POWERS.len(),
        }
    }

    fn update_terms(&mut self) {
        for (j, term) in self.terms.iter_mut().enumerate() {
            *term = *term * Codeword::for_power(j);
        }
    }

    fn sum_terms(&self) -> Codeword {
        self.terms.iter().fold(Codeword::default(), |s, &x| {
            s + x
        })
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
                return Some(Codeword::for_power(pow).invert().power().unwrap());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{encode, Syndromes, Codeword, Polynomial, decode};

    #[test]
    fn test_for_power() {
        assert_eq!(Codeword::for_power(0).0, 0b100000);
        assert_eq!(Codeword::for_power(62).0, 0b100001);
        assert_eq!(Codeword::for_power(63).0, 0b100000);
    }

    #[test]
    fn test_add_sub() {
        assert_eq!((Codeword::new(0b100000) + Codeword::new(0b010000)).0, 0b110000);
        assert_eq!((Codeword::new(0b100000) - Codeword::new(0b010000)).0, 0b110000);
        assert_eq!((Codeword::new(0b100001) + Codeword::new(0b100001)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) - Codeword::new(0b100001)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) + Codeword::new(0b110100)).0, 0b010101);
        assert_eq!((Codeword::new(0b100001) - Codeword::new(0b110100)).0, 0b010101);
    }

    #[test]
    fn test_mul() {
        assert_eq!((Codeword::new(0b011000) * Codeword::new(0b101000)).0, 0b011110);
        assert_eq!((Codeword::new(0b000000) * Codeword::new(0b101000)).0, 0b000000);
        assert_eq!((Codeword::new(0b011000) * Codeword::new(0b000000)).0, 0b000000);
        assert_eq!((Codeword::new(0b000000) * Codeword::new(0b000000)).0, 0b000000);
        assert_eq!((Codeword::new(0b100001) * Codeword::new(0b100000)).0, 0b100001);
        assert_eq!((Codeword::new(0b100001) * Codeword::new(0b010000)).0, 0b100000);
        assert_eq!((Codeword::new(0b110011) * Codeword::new(0b110011)).0, 0b100111);
        assert_eq!((Codeword::new(0b111101) * Codeword::new(0b111101)).0, 0b011001);
    }


    #[test]
    fn test_div() {
        assert_eq!((Codeword::new(0b000100) / Codeword::new(0b101000)).0, 0b111010);
        assert_eq!((Codeword::new(0b000000) / Codeword::new(0b101000)).0, 0b000000);
        assert_eq!((Codeword::new(0b011110) / Codeword::new(0b100000)).0, 0b011110);
        assert_eq!((Codeword::new(0b011110) / Codeword::new(0b011110)).0, 0b100000);
    }

    #[test]
    fn test_cmp() {
        assert!(Codeword::new(0b100000) > Codeword::new(0b000000));
        assert!(Codeword::new(0b000000) == Codeword::new(0b000000));
        assert!(Codeword::new(0b010000) > Codeword::new(0b100000));
        assert!(Codeword::new(0b100001) > Codeword::new(0b100000));
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

        assert!(Syndromes::new(w).all(|s| s.zero()));
        assert!(!Syndromes::new(w ^ 1<<60).all(|s| s.zero()));
    }

    #[test]
    fn test_polynomial() {
        let p = Polynomial::new((0..23).map(|i| {
            Codeword::for_power(i)
        }));

        assert!(p.degree().unwrap() == 22);
        assert!(p.constant() == Codeword::for_power(0));

        let p = p.shift();
        assert!(p.degree().unwrap() == 21);
        assert!(p.constant() == Codeword::for_power(1));

        let q = p.clone() * Codeword::for_power(0);
        assert!(q.degree().unwrap() == 21);
        assert!(q.constant() == Codeword::for_power(1));

        let q = p.clone() * Codeword::for_power(2);
        assert!(q.degree().unwrap() == 21);
        assert!(q.constant() == Codeword::for_power(3));

        let q = p.clone() + p.clone();
        assert!(q.constant().zero());

        for coef in q.coefs() {
            assert!(coef.zero());
        }

        let p = Polynomial::new((4..27).map(|i| {
            Codeword::for_power(i)
        }));

        let q = Polynomial::new((3..26).map(|i| {
            Codeword::for_power(i)
        }));

        let r = p + q.shift();

        assert!(r.coefs[0].zero());
        assert!(r.coefs[1].zero());
        assert!(r.coefs[2].zero());
        assert!(r.coefs[3].zero());
        assert!(r.coefs[4].zero());
        assert!(!r.coefs[22].zero());

        let p = Polynomial::new((0..2).map(|_| {
            Codeword::for_power(0)
        }));

        let q = Polynomial::new((0..4).map(|_| {
            Codeword::for_power(1)
        }));

        let r = p + q;

        assert!(r.coef(0) == Codeword::for_power(6));
    }

    #[test]
    fn test_decode() {
        let w = encode(0b1111111100000000) ^ 0b11010011<<30;
        let d = decode(w);

        match d {
            Ok((0b1111111100000000, 5)) => {},
            _ => panic!(),
        }
    }
}
