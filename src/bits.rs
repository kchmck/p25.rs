//! Utilities for packing/unpacking dibits and tribits into/out of bytes.

use std;

/// Iterate over the 2-bit symbols of a byte source, MSB to LSB.
pub type Dibits<T> = SubByteIter<DibitParams, T>;
/// Iterates over the 3-bit symbols of a byte source, MSB to LSB.
pub type Tribits<T> = SubByteIter<TribitParams, T>;
/// Iterates over the 6-bit symbols of a byte source, MSB to LSB. The source must be a
/// multiple of 3 bytes.
pub type Hexbits<T> = SubByteIter<HexbitParams, T>;

/// Groups dibits into full bytes. The source must be a multiple of 4 dibits.
pub type DibitBytes<T> = SubByteIter<DibitByteParams, T>;
/// Groups tribits into full bytes. The source must be a multiple of 8 tribits.
pub type TribitBytes<T> = SubByteIter<TribitByteParams, T>;
/// Groups hexbits into full bytes. The source must be a multiple of 6 hexbits.
pub type HexbitBytes<T> = SubByteIter<HexbitByteParams, T>;

pub trait IterParams {
    /// Type to consume when buffering.
    type Input;
    /// Type to yield at each iteration.
    type Output;

    /// Number of bits to consume at each iteration.
    fn bits() -> usize;

    /// Number of input symbols to consume when buffering.
    fn buffer() -> usize;

    /// Amount to shift buffer after loading an input symbol.
    fn shift() -> usize;

    /// Amount to shift buffer after all buffering.
    fn post_shift() -> usize { 32 - Self::shift() * Self::buffer() }

    /// Number of iterations before buffering.
    fn iterations() -> usize { Self::shift() * Self::buffer() / Self::bits() }

    /// Convert input symbol to a byte.
    fn to_byte(input: Self::Input) -> u8;

    /// Convert bits to output type.
    fn to_output(bits: u8) -> Self::Output;

    /// Verify the parameters are supported.
    fn validate() {
        // Maximum buffer size is currently 32 bits.
        assert!(Self::buffer() * Self::shift() <= 32);
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

    /// Get the wrapped dibit, which is guaranteed to have only 2 LSBs.
    pub fn bits(&self) -> u8 { self.0 }
    /// Get the MSB.
    pub fn hi(&self) -> u8 { self.0 >> 1 }
    /// Get the LSB.
    pub fn lo(&self) -> u8 { self.0 & 1 }
}

/// Parameters for `Dibits` iterator.
pub struct DibitParams;

impl IterParams for DibitParams {
    type Input = u8;
    type Output = Dibit;

    fn bits() -> usize { 2 }
    fn buffer() -> usize { 1 }
    fn shift() -> usize { 8 }

    fn to_byte(input: Self::Input) -> u8 { input }
    fn to_output(bits: u8) -> Dibit { Dibit::new(bits) }
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

    /// Get the wrapped tribit, which is guaranteed to have only 3 LSBs.
    pub fn bits(&self) -> u8 { self.0 }
}

/// Parameters for `Tribits` iterator.
pub struct TribitParams;

impl IterParams for TribitParams {
    type Input = u8;
    type Output = Tribit;

    fn bits() -> usize { 3 }
    fn buffer() -> usize { 3 }
    fn shift() -> usize { 8 }

    fn to_byte(input: Self::Input) -> u8 { input }
    fn to_output(bits: u8) -> Tribit { Tribit::new(bits) }
}

/// Six bits.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Hexbit(u8);

impl Hexbit {
    /// Construct a new `Hexbit` with the 6 given bits in the LSB position.
    pub fn new(bits: u8) -> Hexbit {
        assert!(bits >> 6 == 0);
        Hexbit(bits)
    }

    /// Get the wrapped hexbit, which is guaranteed to have only 6 LSBs.
    pub fn bits(&self) -> u8 { self.0 }
}

/// Parameters for `Hexbits` iterator.
pub struct HexbitParams;

impl IterParams for HexbitParams {
    type Input = u8;
    type Output = Hexbit;

    fn bits() -> usize { 6 }
    fn buffer() -> usize { 3 }
    fn shift() -> usize { 8 }

    fn to_byte(input: Self::Input) -> u8 { input }
    fn to_output(bits: u8) -> Hexbit { Hexbit::new(bits) }
}

/// Parameters for `DibitBytes` iterator.
pub struct DibitByteParams;

impl IterParams for DibitByteParams {
    type Input = Dibit;
    type Output = u8;

    fn bits() -> usize { 8 }
    fn buffer() -> usize { 4 }
    fn shift() -> usize { 2 }

    fn to_byte(input: Self::Input) -> u8 { input.bits() }
    fn to_output(bits: u8) -> Self::Output { bits }
}

/// Parameters for `TribitBytes` iterator.
pub struct TribitByteParams;

impl IterParams for TribitByteParams {
    type Input = Tribit;
    type Output = u8;

    fn bits() -> usize { 8 }
    fn buffer() -> usize { 8 }
    fn shift() -> usize { 3 }

    fn to_byte(input: Self::Input) -> u8 { input.bits() }
    fn to_output(bits: u8) -> Self::Output { bits }
}

/// Parameters for `HexbitBytes` iterator.
pub struct HexbitByteParams;

impl IterParams for HexbitByteParams {
    type Input = Hexbit;
    type Output = u8;

    fn bits() -> usize { 8 }
    fn buffer() -> usize { 4 }
    fn shift() -> usize { 6 }

    fn to_byte(input: Self::Input) -> u8 { input.bits() }
    fn to_output(bits: u8) -> Self::Output { bits }
}

/// An iterator for sub-byte (bit-level) values.
pub struct SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = P::Input>
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
    P: IterParams, T: Iterator<Item = P::Input>
{
    /// Construct a new `SubByteIter` over the given symbol source.
    pub fn new(src: T) -> SubByteIter<P, T> {
        SubByteIter {
            params: std::marker::PhantomData,
            src: src,
            buf: 0,
            idx: 0,
        }
    }

    /// Consume one or more symbols to create a buffer of bits, filled starting from the
    /// MSB.
    fn buffer(&mut self) -> Option<u32> {
        let (buf, added) = (&mut self.src)
            .take(P::buffer())
            .fold((0, 0), |(buf, added), next| {
                (buf << P::shift() | P::to_byte(next) as u32, added + 1)
            });

        // It's okay if there are no more source symbols here, because we're on a safe
        // boundary.
        if added == 0 {
            return None;
        }

        assert!(added == P::buffer(), "incomplete source");

        Some(buf << P::post_shift())
    }
}

impl<P, T> Iterator for SubByteIter<P, T> where
    P: IterParams, T: Iterator<Item = P::Input>
{
    type Item = P::Output;

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

        Some(P::to_output(bits as u8))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn validate_params() {
        DibitParams::validate();
        TribitParams::validate();
        HexbitParams::validate();
        DibitByteParams::validate();
        TribitByteParams::validate();
        HexbitByteParams::validate();
    }

    #[test]
    fn test_dibits() {
        let bytes = [
            0b00110011,
            0b10011001,
            0b11111111,
        ];

        let mut d = Dibits::new(bytes.iter().cloned());

        assert_eq!(d.next().unwrap().bits(), 0b00);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert_eq!(d.next().unwrap().bits(), 0b00);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert_eq!(d.next().unwrap().bits(), 0b10);
        assert_eq!(d.next().unwrap().bits(), 0b01);
        assert_eq!(d.next().unwrap().bits(), 0b10);
        assert_eq!(d.next().unwrap().bits(), 0b01);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert_eq!(d.next().unwrap().bits(), 0b11);
        assert!(d.next().is_none());
    }

    #[test]
    fn test_dibit_bytes() {
        let dibits = [
            Dibit::new(0b00),
            Dibit::new(0b11),
            Dibit::new(0b00),
            Dibit::new(0b11),
            Dibit::new(0b10),
            Dibit::new(0b01),
            Dibit::new(0b10),
            Dibit::new(0b01),
            Dibit::new(0b11),
            Dibit::new(0b11),
            Dibit::new(0b11),
            Dibit::new(0b11),
        ];

        let mut d = DibitBytes::new(dibits.iter().cloned());

        assert_eq!(d.next().unwrap(), 0b00110011);
        assert_eq!(d.next().unwrap(), 0b10011001);
        assert_eq!(d.next().unwrap(), 0b11111111);
        assert!(d.next().is_none());
    }

    #[test]
    #[should_panic]
    fn test_dibit_bytes_panic() {
        let dibits = [
            Dibit::new(0b00),
            Dibit::new(0b11),
            Dibit::new(0b00),
            Dibit::new(0b11),
            Dibit::new(0b10),
        ];

        let mut d = DibitBytes::new(dibits.iter().cloned());

        d.next();
        d.next();
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

    #[test]
    fn test_tribit_bytes() {
        let tribits = [
            Tribit::new(0b001),
            Tribit::new(0b010),
            Tribit::new(0b011),
            Tribit::new(0b100),
            Tribit::new(0b101),
            Tribit::new(0b110),
            Tribit::new(0b111),
            Tribit::new(0b000),
            Tribit::new(0b001),
            Tribit::new(0b010),
            Tribit::new(0b011),
            Tribit::new(0b100),
            Tribit::new(0b101),
            Tribit::new(0b110),
            Tribit::new(0b111),
            Tribit::new(0b000),
        ];

        let mut t = TribitBytes::new(tribits.iter().cloned());

        assert_eq!(t.next().unwrap(), 0b00101001);
        assert_eq!(t.next().unwrap(), 0b11001011);
        assert_eq!(t.next().unwrap(), 0b10111000);
        assert_eq!(t.next().unwrap(), 0b00101001);
        assert_eq!(t.next().unwrap(), 0b11001011);
        assert_eq!(t.next().unwrap(), 0b10111000);
        assert!(t.next().is_none());
    }

    #[test]
    #[should_panic]
    fn test_tribit_bytes_panic() {
        let tribits = [
            Tribit::new(0b001),
            Tribit::new(0b010),
            Tribit::new(0b011),
            Tribit::new(0b100),
        ];

        let mut t = TribitBytes::new(tribits.iter().cloned());

        t.next();
        t.next();
    }

    #[test]
    fn test_hexbits() {
        let bytes = [
            0b11111100,
            0b00001010,
            0b10010101,
            0b11111100,
            0b00001010,
            0b10010101,
        ];

        let mut h = Hexbits::new(bytes.iter().cloned());

        assert_eq!(h.next().unwrap().bits(), 0b111111);
        assert_eq!(h.next().unwrap().bits(), 0b000000);
        assert_eq!(h.next().unwrap().bits(), 0b101010);
        assert_eq!(h.next().unwrap().bits(), 0b010101);
        assert_eq!(h.next().unwrap().bits(), 0b111111);
        assert_eq!(h.next().unwrap().bits(), 0b000000);
        assert_eq!(h.next().unwrap().bits(), 0b101010);
        assert_eq!(h.next().unwrap().bits(), 0b010101);
        assert!(h.next().is_none());
    }

    #[test]
    #[should_panic]
    fn test_hexbits_panic() {
        let bytes = [
            0b11111100,
            0b00001010,
        ];

        let mut h = Hexbits::new(bytes.iter().cloned());

        assert_eq!(h.next().unwrap().bits(), 0b111111);
        assert_eq!(h.next().unwrap().bits(), 0b000000);
        h.next();
    }

    #[test]
    fn test_hexbit_bytes() {
        let hexbits = [
            Hexbit::new(0b111111),
            Hexbit::new(0b000000),
            Hexbit::new(0b101010),
            Hexbit::new(0b010101),
            Hexbit::new(0b111111),
            Hexbit::new(0b000000),
            Hexbit::new(0b101010),
            Hexbit::new(0b010101),
        ];

        let mut h = HexbitBytes::new(hexbits.iter().cloned());

        assert_eq!(h.next().unwrap(), 0b11111100);
        assert_eq!(h.next().unwrap(), 0b00001010);
        assert_eq!(h.next().unwrap(), 0b10010101);
        assert_eq!(h.next().unwrap(), 0b11111100);
        assert_eq!(h.next().unwrap(), 0b00001010);
        assert_eq!(h.next().unwrap(), 0b10010101);
        assert!(h.next().is_none());
    }

    #[test]
    #[should_panic]
    fn test_hexbit_bytes_panic() {
        let hexbits = [
            Hexbit::new(0b111111),
            Hexbit::new(0b000000),
            Hexbit::new(0b101010),
            Hexbit::new(0b010101),
            Hexbit::new(0b111111),
        ];

        let mut h = HexbitBytes::new(hexbits.iter().cloned());
        h.next();
        h.next();
        h.next();
        h.next();
        h.next();
        h.next();
    }
}
