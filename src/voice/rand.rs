//! Pseudo-random (PN) sequence used for voice frame scrambling/descrambling.

/// Generates 23-bit and 15-bit scrambling words using the P25 PN sequence algorithm.
pub struct PseudoRand {
    /// Current state, known as `p_n` in the standard.
    state: u16,
}

impl PseudoRand {
    /// Create a new `PseudoRand` generator using the given 12-bit seed.
    pub fn new(init: u16) -> PseudoRand {
        assert!(init >> 12 == 0);

        PseudoRand {
            state: init << 4,
        }
    }

    /// Retrieve the next 23-bit scrambling word.
    pub fn next_23(&mut self) -> u32 { self.next_bits(23) }
    /// Retrieve the next 15-bit scrambling word.
    pub fn next_15(&mut self) -> u32 { self.next_bits(15) }

    /// Generate a scrambling word of the given size, up to 32 bits. The resulting word
    /// has `p_n(15)` as MSB and `p_{n+bits-1}(15)` as LSB, in the notation of the
    /// standard.
    fn next_bits(&mut self, bits: usize) -> u32 {
        assert!(bits <= 32);

        // Continously shift random bits into LSB.
        (0..bits).fold(0, |buf, _| {
            buf << 1 | self.advance() as u32
        })
    }

    /// Step the generator and retrieve the next random bit.
    fn advance(&mut self) -> u16 {
        self.state = self.next_state();
        self.next_bit()
    }

    /// Retrieve the random bit for the current state (defined as the MSB, `p_n(15)`, by
    /// the standard.)
    fn next_bit(&self) -> u16 {
        self.state >> 15
    }

    /// Step to the next state using the formula in the standard. Since the containing
    /// type is 16 bits, the modulo operation is implicit.
    fn next_state(&self) -> u16 {
        self.state.wrapping_mul(173).wrapping_add(13849)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_prand() {
        let mut prand = PseudoRand::new(0xABC);

        assert_eq!(prand.next_state(), 18137);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 5822);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 38015);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 36844);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 30869);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 45770);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 2203);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 1752);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 54801);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 57238);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 20087);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 15492);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 6989);
        assert_eq!(prand.advance(), 0);
        assert_eq!(prand.next_state(), 43298);
        assert_eq!(prand.advance(), 1);
        assert_eq!(prand.next_state(), 33299);
        assert_eq!(prand.advance(), 1);

        let mut prand = PseudoRand::new(0xABC);
        assert_eq!(prand.next_15(), 0b001101001100011);
    }
}
