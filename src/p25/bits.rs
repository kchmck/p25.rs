//! This module defines the `Bits`, `Dibits`, and `Tribits` iterators as well as the
//! wrapper types `Bit`, `Dibit`, and `Tribit`, for working with sub-byte values.
//!
//! The wrapped values of `Bit`, `Dibit`, and `Tribit, are guaranteed to have only one,
//! two, or three bits, respectively.

use std;

/// Iterate over the dibits of a byte source, MSB to LSB.
pub type Dibits<T> = SubByteIter<DibitParams, T>;
/// Iterates over the tribits in a byte source, MSB to LSB.
pub type Tribits<T> = SubByteIter<TribitParams, T>;

/// Defines parameters needed for (power of two) sub-byte iterators.
pub trait IterParams {
    /// Type to yield at each iteration.
    type IterType;

    /// Number of bits to consume at each iteration.
    fn bits() -> usize;

    /// Number of bytes to buffer, where the number of bits contained should be the lcm of
    /// 8 and the number of bits per iteration.
    fn buffer() -> usize { 1 }

    /// Amount to shift buffer after loading in bytes.
    fn buffer_shift() -> usize { 32 - 8 * Self::buffer() }

    /// Number of iterations needed for each byte.
    fn iterations() -> usize { 8 * Self::buffer() / Self::bits() }

    /// Wrap the given bits in container type.
    fn wrap(bits: u8) -> Self::IterType;

    /// Verify the parameters are supported.
    fn validate() {
        // Maximum buffer size is currently 32 bits.
        assert!(Self::buffer() <= 4);
    }
}

/// Two bits.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Dibit(u8);

impl Dibit {
    /// Construct a new `Dibit` with the two given bits in the LSB position.
    pub fn new(bits: u8) -> Dibit {
        assert!(bits >> 2 == 0);
        Dibit(bits)
    }

    /// Get the wrapped dibit.
    pub fn bits(&self) -> u8 { self.0 }
}

/// Parameters for `Dibits` iterator.
pub struct DibitParams;

impl IterParams for DibitParams {
    type IterType = Dibit;

    fn bits() -> usize { 2 }
    fn wrap(bits: u8) -> Dibit { Dibit::new(bits) }
}

/// Three bits.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Tribit(u8);

impl Tribit {
    /// Construct a new `Tribit` with the three given bits in the LSB position.
    pub fn new(bits: u8) -> Tribit {
        assert!(bits >> 3 == 0);
        Tribit(bits)
    }

    /// Get the wrapped tribit.
    pub fn bits(&self) -> u8 { self.0 }
}

/// Parameters for `Tribits` iterator.
pub struct TribitParams;

impl IterParams for TribitParams {
    type IterType = Tribit;

    fn bits() -> usize { 3 }
    fn buffer() -> usize { 3 }
    fn wrap(bits: u8) -> Tribit { Tribit::new(bits) }
}

/// An iterator for sub-byte (bit-level) values.
struct SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = u8>
{
    params: std::marker::PhantomData<P>,
    /// Source of bytes.
    src: T,
    /// Current buffered bits.
    buf: u32,
    /// Current bit-level index into the current byte.
    idx: u8,
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
            buf: 0,
            idx: 0,
        }
    }

    /// Consume one or more bytes to create a buffer of bits, filled starting from the
    /// MSB.
    fn buffer(&mut self) -> Option<u32> {
        let (buf, added) = (&mut self.src)
            .take(P::buffer())
            .fold((0, 0), |(buf, added), byte| {
                (buf << 8 | byte as u32, added + 1)
            });

        // It's okay if there are no more source bits here, because we're on a safe
        // boundary.
        if added == 0 {
            return None;
        }

        assert!(added == P::buffer(), "incomplete source");

        Some(buf << P::buffer_shift())
    }
}

impl<P, T> Iterator for SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = u8>
{
    type Item = P::IterType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.buf = match self.buffer() {
                Some(b) => b,
                None => return None,
            };
        }

        // Extract MSBs.
        let bits = self.buf >> (32 - P::bits());

        // Strip off the MSBs for the next iteration.
        self.buf <<= P::bits();

        // Move to the next item and reset after all have been visited.
        self.idx += 1;
        self.idx %= P::iterations() as u8;

        Some(P::wrap(bits as u8))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_params() {
        DibitParams::validate();
        TribitParams::validate();
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
