pub struct PseudoRand {
    state: u16,
}

impl PseudoRand {
    pub fn new(init: u16) -> PseudoRand {
        assert!(init >> 12 == 0);

        PseudoRand {
            state: init << 4,
        }
    }

    pub fn next_23(&mut self) -> u32 { self.next_bits(23) }
    pub fn next_15(&mut self) -> u32 { self.next_bits(15) }

    fn next_bits(&mut self, bits: usize) -> u32 {
        assert!(bits <= 32);

        (0..bits).fold(0, |buf, _| {
            buf << 1 | self.advance() as u32
        })
    }

    fn advance(&mut self) -> u16 {
        self.state = self.next_state();
        self.next_bit()
    }

    fn next_bit(&self) -> u16 {
        self.state >> 15
    }

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
