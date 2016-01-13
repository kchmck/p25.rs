//! Implements the 9, 16, and 32-bit CRCs defined by P25 for data checksums.
//!
//! This implementation uses the typical long division and takes advantage of the short
//! lengths to use only a 64-bit word as a buffer, allowing simple bitwise operations.

use std;

/// 9-bit CRC calculator.
pub type CRC9 = CRC<CRC9Params>;

/// 16-bit CRC calculator.
pub type CRC16 = CRC<CRC16Params>;

/// 32-bit CRC calculator.
pub type CRC32 = CRC<CRC32Params>;

pub trait CRCParams {
    /// Generator polynomial, with the MSB being the coefficient of highest degree.
    fn gen() -> u64;

    /// Inversion polynomial, with the MSB being the coefficient of highest degree.
    fn inv() -> u64;

    /// Amount to left-shift the message (multiply by x^i) before division.
    fn shift() -> usize;

    /// Verify the parameters are well-formed.
    fn validate() {
        // Prevent division by zero.
        assert!(Self::gen() != 0);
        // Ensure the generator can be left-shifted by up to a byte (since that's the
        // maximum number of bits that will be fed in per long division step.)
        assert!(degree(Self::gen()) < 64 - 8);
    }
}

/// Params for 9-bit CRC.
struct CRC9Params;

impl CRCParams for CRC9Params {
    fn gen() -> u64 { 0b1001011001 }
    fn inv() -> u64 { 0b111111111 }
    fn shift() -> usize { 9 }
}

/// Params for 16-bit CRC.
struct CRC16Params;

impl CRCParams for CRC16Params {
    fn gen() -> u64 { 0b10001000000100001 }
    fn inv() -> u64 { 0b1111111111111111 }
    fn shift() -> usize { 16 }
}

/// Params for 32-bit CRC.
struct CRC32Params;

impl CRCParams for CRC32Params {
    fn gen() -> u64 { 0b100000100110000010001110110110111 }
    fn inv() -> u64 { 0b11111111111111111111111111111111 }
    fn shift() -> usize { 32 }
}

/// CRC calculator using long division.
struct CRC<P: CRCParams> {
    params: std::marker::PhantomData<P>,
    /// Current output of the calculator.
    word: u64
}

impl<P: CRCParams> CRC<P> {
    /// Construct a new `CRC` with empty output.
    pub fn new() -> CRC<P> {
        CRC {
            params: std::marker::PhantomData,
            word: 0,
        }
    }

    /// Feed in `num` LSBs of the given byte.
    pub fn feed_bits(&mut self, bits: u8, num: usize) -> &mut Self {
        assert!(num <= 8);
        // Verify there are no stray MSBs.
        assert!((bits as u16) >> num == 0);

        self.word <<= num;
        self.word |= bits as u64;

        self.div();
        self
    }

    /// Feed in the given byte stream.
    pub fn feed_bytes<T: IntoIterator<Item = u8>>(&mut self, bytes: T) -> &mut Self {
        for byte in bytes {
            self.feed_bits(byte, 8);
        }

        self
    }

    /// Finish the CRC calculation and return the resulting CRC.
    pub fn finish(&mut self) -> u64 {
        self.flush();
        self.word ^ P::inv()
    }

    /// Reduce the current word by dividing by the generator.
    fn div(&mut self) {
        while self.word != 0 {
            let diff = degree(self.word) as i32 - degree(P::gen()) as i32;

            // If the divisor (generator) has higher degree than the dividend (word), then
            // no more division can be done.
            if diff < 0 {
                break;
            }

            // Bring the generator up to the same degree and knock off at least one of the
            // word's MSBs.
            self.word ^= P::gen() << diff;
        }
    }

    /// Perform the final shift and division of the word.
    fn flush(&mut self) {
        for _ in 0..P::shift() {
            self.word <<= 1;
            self.div();
        }
    }
}

// Calculate the degree of the polynomial represented by x, where x > 0.
fn degree(x: u64) -> u32 {
    64 - 1 - x.leading_zeros()
}

#[cfg(test)]
mod test {
    use super::*;
    use super::{CRC9Params, CRC16Params, CRC32Params, CRC};

    struct CRCTest;

    impl CRCParams for CRCTest {
        fn gen() -> u64 { 0b100011011 }
        fn inv() -> u64 { 0b111 }
        fn shift() -> usize { 0 }
    }

    struct CRCTestShifted;

    impl CRCParams for CRCTestShifted {
        fn gen() -> u64 { 0b10001101100 }
        fn inv() -> u64 { 0b111 }
        fn shift() -> usize { 2 }
    }

    #[test]
    fn validate_params() {
        CRC9Params::validate();
        CRC16Params::validate();
        CRC32Params::validate();
    }

    #[test]
    fn test_calc() {
        let mut c = CRC::<CRCTest>::new();
        c.feed_bytes([
            0b00111111,
            0b01111110,
        ].iter().cloned());
        assert_eq!(c.finish(), 0b110);
    }

    #[test]
    fn test_shift() {
        assert_eq!(CRC::<CRCTestShifted>::new().feed_bytes([
            0b00111111,
            0b01111110,
        ].iter().cloned()).finish(), 0b011);
    }

    #[test]
    fn test_crc32() {
        assert_eq!(CRC32::new().feed_bytes([
            0b1010,
        ].iter().cloned()).finish(),
        0b11010000011101010010100100101001);
    }
}
