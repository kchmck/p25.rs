//! This module defines the `Bits`, `Dibits`, and `Tribits` iterators as well as the
//! wrapper types `Bit`, `Dibit`, and `Tribit`, for working with sub-byte values.
//!
//! The wrapped values of `Bit`, `Dibit`, and `Tribit, are guaranteed to have only one,
//! two, or three bits, respectively.

use std;

/// Iterate over individual bits of a byte source, MSB to LSB.
pub type Bits<T> = SubByteIter<BitParams, T>;
/// Iterate over the dibits of a byte source, MSB to LSB.
pub type Dibits<T> = SubByteIter<DibitParams, T>;

/// Defines parameters needed for (power of two) sub-byte iterators.
pub trait IterParams {
    /// Type to yield at each iteration.
    type IterType;

    /// Number of bits to consume at each iteration.
    fn bits() -> u8;
    /// Wrap the given bits in container type.
    fn wrap(bits: u8) -> Self::IterType;

    /// Number of iterations needed for each byte.
    fn iterations() -> u8 { 8 / Self::bits() }

    /// Verify the parameters are supported.
    fn validate() {
        // Only powers of two are valid because there can be no "leftovers."
        assert!(Self::bits().is_power_of_two());
    }
}

/// A single bit.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Bit(u8);

impl Bit {
    /// Construct a new `Bit` with the given bit in the LSB position.
    pub fn new(bits: u8) -> Bit {
        assert!(bits & 0b11111110 == 0);
        Bit(bits)
    }

    /// Get the wrapped bit value.
    pub fn bit(&self) -> u8 { self.0 }
}

/// Parameters for `Bits` iterator.
pub struct BitParams;

impl IterParams for BitParams {
    type IterType = Bit;
    fn bits() -> u8 { 1 }
    fn wrap(bits: u8) -> Bit { Bit::new(bits) }
}

/// Two bits.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Dibit(u8);

impl Dibit {
    /// Construct a new `Dibit` with the two given bits in the LSB position.
    pub fn new(bits: u8) -> Dibit {
        assert!(bits & 0b11111100 == 0);
        Dibit(bits)
    }

    /// Get the wrapped dibit.
    pub fn bits(&self) -> u8 { self.0 }
}

/// Parameters for `Dibits` iterator.
pub struct DibitParams;

impl IterParams for DibitParams {
    type IterType = Dibit;
    fn bits() -> u8 { 2 }
    fn wrap(bits: u8) -> Dibit { Dibit::new(bits) }
}

/// Three bits.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Tribit(u8);

impl Tribit {
    /// Construct a new `Tribit` with the three given bits in the LSB position.
    pub fn new(bits: u8) -> Tribit {
        assert!(bits & 0b11111000 == 0);
        Tribit(bits)
    }

    /// Get the wrapped tribit.
    pub fn bits(&self) -> u8 { self.0 }
}

/// An iterator for sub-byte (bit-level) values.
struct SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = u8>
{
    params: std::marker::PhantomData<P>,
    /// Source of bytes.
    src: T,
    /// Current bit-level index into the current byte.
    idx: u8,
    /// Current byte in the source.
    byte: u8,
}

impl<P, T> SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = u8>
{
    /// Construct a new `SubByteIter` over the given byte source. All bits are iterated
    /// over, so the number of bits must be a byte multiple.
    pub fn new(src: T) -> SubByteIter<P, T> {
        SubByteIter {
            params: std::marker::PhantomData,
            src: src,
            byte: 0,
            idx: 0,
        }
    }
}

impl<P, T> Iterator for SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = u8>
{
    type Item = P::IterType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.byte = match self.src.next() {
                Some(b) => b,
                None => return None,
            };
        }

        // Extract MSBs.
        let bits = self.byte >> (8 - P::bits());

        // Strip off the MSBs for the next iteration.
        self.byte <<= P::bits();

        // Move to the next item and reset after all have been visited.
        self.idx += 1;
        self.idx %= P::iterations();

        Some(P::wrap(bits))
    }
}

/// Iterates over the tribits in a byte source.
pub struct Tribits<T: Iterator<Item = u8>> {
    /// The source of bytes.
    src: T,
    /// Current bit buffer, containing either 6 or 3 bits (in MSB position) or 0.
    bits: u8,
    /// Tribit index into `bits` (0 or 1).
    idx: usize,
    /// Buffered bits from current source byte, to be added to `bits`.
    buf: u8,
    /// Number of bits in `buf`, either 2, 4, 6, or 0.
    buf_bits: usize,
}

impl<T: Iterator<Item = u8>> Tribits<T> {
    /// Construct a new `Tribits` from the given source of bytes. The number of bytes must
    /// be a multiple of 3 (a multiple of 24 bits).
    pub fn new(src: T) -> Tribits<T> {
        Tribits {
            src: src,
            bits: 0,
            idx: 0,
            buf: 0,
            buf_bits: 0,
        }
    }
}

impl<T: Iterator<Item = u8>> Iterator for Tribits<T> {
    type Item = Tribit;

    fn next(&mut self) -> Option<Self::Item> {
        // If on the first tribit, it's time to flush the buffer and (maybe) load another
        // byte.
        if self.idx == 0 {
            // Flush and reset the buffer.
            self.bits = self.buf;
            self.buf = 0;

            // Calculate the number of bits to buffer for the next iteration.
            self.buf_bits += 2;
            self.buf_bits %= 8;

            // Only load a new byte if bits need to be buffered.
            if self.buf_bits != 0 {
                let next = match self.src.next() {
                    Some(b) => b,
                    None => if self.buf_bits == 2 {
                        // In this case we've covered 8 tribits = 24 bits = 3 bytes
                        // exactly, so it's fine if there are no more bytes.
                        return None;
                    } else {
                        panic!("incomplete tribit");
                    }
                };

                // Add in some source bits after the MSBs.
                self.bits |= next >> self.buf_bits << 2;
                // Buffer the rest of the bits.
                self.buf = next << (8 - self.buf_bits);
            }
        }

        // Extract the 3 MSBs and strip them off for next time.
        let bits = self.bits >> 5;
        self.bits <<= 3;

        self.idx += 1;
        self.idx %= 2;

        Some(Tribit::new(bits))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_params() {
        BitParams::validate();
        DibitParams::validate();
    }

    #[test]
    fn test_bits() {
        const BITS: &'static [u8] = &[
            0b00011011,
            0b11001100,
        ];

        {
            let mut d = Dibits::new(BITS.iter().cloned());

            assert!(d.next().unwrap().0 == 0b00);
            assert!(d.next().unwrap().0 == 0b01);
            assert!(d.next().unwrap().0 == 0b10);
            assert!(d.next().unwrap().0 == 0b11);
            assert!(d.next().unwrap().0 == 0b11);
            assert!(d.next().unwrap().0 == 0b00);
            assert!(d.next().unwrap().0 == 0b11);
            assert!(d.next().unwrap().0 == 0b00);
            assert!(d.next().is_none());
        }

        {
            let mut b = Bits::new(BITS.iter().cloned());

            assert!(b.next().unwrap().0 == 0);
            assert!(b.next().unwrap().0 == 0);
            assert!(b.next().unwrap().0 == 0);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 0);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 1);
            assert!(b.next().unwrap().0 == 0);
            assert!(b.next().unwrap().0 == 0);
        }
    }

    #[test]
    fn test_tribits() {
        let bytes = [
            0b00101001,
            0b11001011,
            0b10111000,
            0b00101001,
            0b11001011,
            0b10111000,
        ];
        let mut t = Tribits::new(bytes.iter().cloned());

        assert_eq!(t.next().unwrap().bits(), 0b001);
        assert_eq!(t.next().unwrap().bits(), 0b010);
        assert_eq!(t.next().unwrap().bits(), 0b011);
        assert_eq!(t.next().unwrap().bits(), 0b100);
        assert_eq!(t.next().unwrap().bits(), 0b101);
        assert_eq!(t.next().unwrap().bits(), 0b110);
        assert_eq!(t.next().unwrap().bits(), 0b111);
        assert_eq!(t.next().unwrap().bits(), 0b000);
        assert_eq!(t.next().unwrap().bits(), 0b001);
        assert_eq!(t.next().unwrap().bits(), 0b010);
        assert_eq!(t.next().unwrap().bits(), 0b011);
        assert_eq!(t.next().unwrap().bits(), 0b100);
        assert_eq!(t.next().unwrap().bits(), 0b101);
        assert_eq!(t.next().unwrap().bits(), 0b110);
        assert_eq!(t.next().unwrap().bits(), 0b111);
        assert_eq!(t.next().unwrap().bits(), 0b000);
        assert!(t.next().is_none());
    }

    #[test]
    #[should_panic]
    fn test_tribits_panic() {
        let bytes = [1, 2, 3, 4];
        let t = Tribits::new(bytes.iter().cloned());

        for _ in t {}
    }
}
