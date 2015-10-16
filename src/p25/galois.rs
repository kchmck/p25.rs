use std;

/// GF(2^6) field characterized by α^6+α+1.as described in P25 specification.
#[derive(Copy, Clone, Debug)]
pub struct P25Field;

impl GaloisField for P25Field {
    fn size() -> usize { 63 }

    fn codeword(pow: usize) -> u8 {
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
            0b100001,
        ];

        CODEWORDS[pow]
    }

    fn power(codeword: usize) -> usize {
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

#[derive(Copy, Clone, Debug)]
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

impl<F: GaloisField> std::cmp::PartialOrd for Codeword<F> {
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

impl<F: GaloisField> std::cmp::Ord for Codeword<F> {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_for_power() {
        assert_eq!(P25Codeword::for_power(0), 0b100000);
        assert_eq!(P25Codeword::for_power(62), 0b100001);
        assert_eq!(P25Codeword::for_power(63), 0b100000);
    }

    #[test]
    fn test_add_sub() {
        assert_eq!((P25Codeword::new(0b100000) + P25Codeword::new(0b010000)), 0b110000);
        assert_eq!((P25Codeword::new(0b100000) - P25Codeword::new(0b010000)), 0b110000);
        assert_eq!((P25Codeword::new(0b100001) + P25Codeword::new(0b100001)), 0b000000);
        assert_eq!((P25Codeword::new(0b100001) - P25Codeword::new(0b100001)), 0b000000);
        assert_eq!((P25Codeword::new(0b100001) + P25Codeword::new(0b110100)), 0b010101);
        assert_eq!((P25Codeword::new(0b100001) - P25Codeword::new(0b110100)), 0b010101);
    }

    #[test]
    fn test_mul() {
        assert_eq!((P25Codeword::new(0b011000) * P25Codeword::new(0b101000)), 0b011110);
        assert_eq!((P25Codeword::new(0b000000) * P25Codeword::new(0b101000)), 0b000000);
        assert_eq!((P25Codeword::new(0b011000) * P25Codeword::new(0b000000)), 0b000000);
        assert_eq!((P25Codeword::new(0b000000) * P25Codeword::new(0b000000)), 0b000000);
        assert_eq!((P25Codeword::new(0b100001) * P25Codeword::new(0b100000)), 0b100001);
        assert_eq!((P25Codeword::new(0b100001) * P25Codeword::new(0b010000)), 0b100000);
        assert_eq!((P25Codeword::new(0b110011) * P25Codeword::new(0b110011)), 0b100111);
        assert_eq!((P25Codeword::new(0b111101) * P25Codeword::new(0b111101)), 0b011001);
    }


    #[test]
    fn test_div() {
        assert_eq!((P25Codeword::new(0b000100) / P25Codeword::new(0b101000)), 0b111010);
        assert_eq!((P25Codeword::new(0b000000) / P25Codeword::new(0b101000)), 0b000000);
        assert_eq!((P25Codeword::new(0b011110) / P25Codeword::new(0b100000)), 0b011110);
        assert_eq!((P25Codeword::new(0b011110) / P25Codeword::new(0b011110)), 0b100000);
    }

    #[test]
    fn test_cmp() {
        assert!(P25Codeword::new(0b100000) > P25Codeword::new(0b000000));
        assert!(P25Codeword::new(0b000000) == P25Codeword::new(0b000000));
        assert!(P25Codeword::new(0b010000) > P25Codeword::new(0b100000));
        assert!(P25Codeword::new(0b100001) > P25Codeword::new(0b100000));
    }

    #[test]
    fn test_pow() {
        assert_eq!(P25Codeword::for_power(0).pow(10).power().unwrap(), 0);
        assert_eq!(P25Codeword::for_power(1).pow(10).power().unwrap(), 10);
        assert_eq!(P25Codeword::for_power(62).pow(10).power().unwrap(), 53);
        assert!(P25Codeword::default().pow(20).power().is_none());
    }
}
