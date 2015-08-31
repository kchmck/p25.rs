/// The inner byte has only the two least significant bits populated.
pub struct Dibit(pub u8);

/// Yield a series of dibits (two-bit symbols) from the bytes in the given source,
/// most-siginicant bit to least-significant bit. The number of source bits must be a
/// multiple of 8, i.e., all the bits in all the source bytes are yielded.
pub struct Dibits<T: Iterator<Item = u8>> {
    /// The source of bits.
    src: T,
    /// The current dibit index into the current byte.
    idx: u8,
    /// The current byte in the source.
    byte: u8,
}

impl<T: Iterator<Item = u8>> Dibits<T> {
    /// Construct a new `Dibits<T>`.
    pub fn new(src: T) -> Dibits<T> {
        Dibits {
            src: src,
            byte: 0,
            idx: 0,
        }
    }
}

impl<T: Iterator<Item = u8>> Iterator for Dibits<T> {
    type Item = Dibit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == 0 {
            self.byte = match self.src.next() {
                Some(b) => b,
                None => return None,
            };
        }

        // Store the current byte for later.
        let byte = self.byte;

        // Strip off the 2 MSBs for the next iteration.
        self.byte <<= 2;

        // Move to the next dibit and reset after all have been visited.
        self.idx += 1;
        self.idx %= 4;

        // Yield the two MSBs.
        Some(Dibit(byte >> 6))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_dibits() {
        use super::*;

        const BITS: &'static [u8] = &[
            0b00011011,
            0b11001100,
        ];

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
}
