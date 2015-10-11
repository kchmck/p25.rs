//! This module defines the `Bits` and `Dibits` iterators as well as the wrapper types
//! `Bit` and `Dibit`, for working with sub-byte values.
//!
//! The wrapped values of `Bit` and `Dibit` are guaranteed to have only one or two bits,
//! respectively.

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

/// An iterator for sub-byte (bit-level) values.
pub struct SubByteIter<P, T> where
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
}
