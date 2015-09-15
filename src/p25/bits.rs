use std;

pub fn iter_dibits<T: Iterator<Item = u8>>(src: T) -> Bits<DibitIterator, T> {
    Bits::new(src)
}

pub fn iter_bits<T: Iterator<Item = u8>>(src: T) -> Bits<BitIterator, T> {
    Bits::new(src)
}

pub trait IterateParams {
    type IterType;

    fn bits() -> u8;
    fn wrap(x: u8) -> Self::IterType;

    fn shift() -> u8 { Self::bits() }
    fn iterations() -> u8 { 8 / Self::bits() }

    fn validate() {
        assert!(Self::bits().is_power_of_two());
    }
}

pub struct Bit(pub u8);
pub struct BitIterator;

impl IterateParams for BitIterator {
    type IterType = Bit;
    fn bits() -> u8 { 1 }
    fn wrap(x: u8) -> Bit { Bit(x) }
}

pub struct Dibit(pub u8);
pub struct DibitIterator;

impl IterateParams for DibitIterator {
    type IterType = Dibit;
    fn bits() -> u8 { 2 }
    fn wrap(x: u8) -> Dibit { Dibit(x) }
}

pub struct Bits<P, T>
    where P: IterateParams, T: Iterator<Item = u8>
{
    /// The source of bits.
    src: T,
    /// The current index into the current byte.
    idx: u8,
    /// The current byte in the source.
    byte: u8,

    _params: std::marker::PhantomData<P>,
}

impl<P, T> Bits<P, T>
    where P: IterateParams, T: Iterator<Item = u8>
{
    /// Construct a new `Bits<T>`.
    fn new(src: T) -> Bits<P, T> {
        P::validate();

        Bits {
            src: src,
            byte: 0,
            idx: 0,
            _params: std::marker::PhantomData,
        }
    }
}

impl<P, T> Iterator for Bits<P, T>
    where P: IterateParams, T: Iterator<Item = u8>
{
    type Item = P::IterType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.byte = match self.src.next() {
                Some(b) => b,
                None => return None,
            };
        }

        // Store the current byte for later.
        let byte = self.byte;

        // Strip off the MSBs for the next iteration.
        self.byte <<= P::shift();

        // Move to the next item and reset after all have been visited.
        self.idx += 1;
        self.idx %= P::iterations();

        // Yield the MSBs.
        Some(P::wrap(byte >> (8 - P::shift())))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_bits() {
        use super::*;

        const BITS: &'static [u8] = &[
            0b00011011,
            0b11001100,
        ];

        {
            let mut d = iter_dibits(BITS.iter().cloned());

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
            let mut b = iter_bits(BITS.iter().cloned());

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
