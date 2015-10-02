use std;

pub trait GaloisField {
    /// Maps the given i in α^i to its codeword.
    fn codeword(pow: usize) -> u8;
    /// Maps the given codeword to i in α^i.
    fn power(codeword: usize) -> usize;
    /// Returns the number of unique codewords in the field.
    fn size() -> usize;

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
