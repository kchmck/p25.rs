use std;

pub type Bits<T> = SubByteIter<BitParams, T>;
pub type Dibits<T> = SubByteIter<DibitParams, T>;

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

pub struct Bit(u8);

impl Bit {
    pub fn new(bits: u8) -> Bit {
        assert!(bits & 0b11111110 == 0);
        Bit(bits)
    }

    pub fn bits(&self) -> u8 { self.0 }
}

pub struct BitParams;

impl IterateParams for BitParams {
    type IterType = Bit;
    fn bits() -> u8 { 1 }
    fn wrap(x: u8) -> Bit { Bit::new(x) }
}

#[derive(Debug)]
pub struct Dibit(u8);

impl Dibit {
    pub fn new(bits: u8) -> Dibit {
        assert!(bits & 0b11111100 == 0);
        Dibit(bits)
    }

    pub fn bits(&self) -> u8 { self.0 }
}

pub struct DibitParams;

impl IterateParams for DibitParams {
    type IterType = Dibit;
    fn bits() -> u8 { 2 }
    fn wrap(x: u8) -> Dibit { Dibit::new(x) }
}

pub struct SubByteIter<P, T>
    where P: IterateParams, T: Iterator<Item = u8>
{
    params: std::marker::PhantomData<P>,
    /// The source of bits.
    src: T,
    /// The current index into the current byte.
    idx: u8,
    /// The current byte in the source.
    byte: u8,
}

impl<P, T> SubByteIter<P, T>
    where P: IterateParams, T: Iterator<Item = u8>
{
    /// Construct a new `SubByteIter<T>`.
    pub fn new(src: T) -> SubByteIter<P, T> {
        SubByteIter {
            params: std::marker::PhantomData,
            src: src,
            byte: 0,
            idx: 0,
        }
    }
}

impl<P, T> Iterator for SubByteIter<P, T>
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
