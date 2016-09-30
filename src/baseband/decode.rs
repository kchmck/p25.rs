use bits;
use consts;

/// Decodes symbol from sample at each symbol instant.
#[derive(Copy, Clone)]
pub struct Decoder {
    pos: usize,
    decider: Decider,
}

impl Decoder {
    /// Create a new Decoder with the given symbol decider.
    pub fn new(decider: Decider) -> Decoder {
        Decoder {
            // Decoder is created after first sample of first symbol after sync has
            // already been read.
            pos: 1,
            decider: decider,
        }
    }

    /// Examine the given sample and, based on the symbol clock, decode it a symbol or
    /// do nothing.
    pub fn feed(&mut self, s: f32) -> Option<bits::Dibit> {
        self.pos += 1;
        self.pos %= consts::PERIOD;

        if self.pos == 0 {
            Some(self.decider.decide(s))
        } else {
            None
        }
    }
}

/// Decides which symbol a sample represents with a threshold method.
#[derive(Copy, Clone)]
pub struct Decider {
    pthresh: f32,
    mthresh: f32,
    nthresh: f32,
}

impl Decider {
    /// Create a new Decider with the given positive threshold, mid threshold, and
    /// negative threshold.
    pub fn new(pthresh: f32, mthresh: f32, nthresh: f32) -> Decider {
        Decider {
            pthresh: pthresh,
            mthresh: mthresh,
            nthresh: nthresh,
        }
    }

    /// Decide with symbol the given sample looks closest to.
    pub fn decide(&self, sample: f32) -> bits::Dibit {
        if sample > self.pthresh {
            bits::Dibit::new(0b01)
        } else if sample > self.mthresh && sample <= self.pthresh {
            bits::Dibit::new(0b00)
        } else if sample <= self.mthresh && sample > self.nthresh {
            bits::Dibit::new(0b10)
        } else {
            bits::Dibit::new(0b11)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decider() {
        let d = Decider::new(-0.004, -0.1, -0.196);

        assert_eq!(d.decide(0.044).bits(), 0b01);
        assert_eq!(d.decide(-0.052).bits(), 0b00);
        assert_eq!(d.decide(-0.148).bits(), 0b10);
        assert_eq!(d.decide(-0.244).bits(), 0b11);
    }

    #[test]
    fn test_decoder() {
        let mut d = Decoder::new(Decider::new(0.0, 0.0, 0.0));

        assert!(d.feed(0.2099609375000000).is_none());
        assert!(d.feed(0.2165222167968750).is_none());
        assert!(d.feed(0.2179870605468750).is_none());
        assert!(d.feed(0.2152709960937500).is_none());
        assert!(d.feed(0.2094726562500000).is_none());
        assert!(d.feed(0.2018737792968750).is_none());
        assert!(d.feed(0.1937255859375000).is_none());
        assert!(d.feed(0.1861572265625000).is_none());
        assert!(d.feed(0.1799926757812500).is_some());

        assert!(d.feed(0.1752929687500000).is_none());
        assert!(d.feed(0.1726684570312500).is_none());
        assert!(d.feed(0.1720886230468750).is_none());
        assert!(d.feed(0.1732177734375000).is_none());
        assert!(d.feed(0.1754455566406250).is_none());
        assert!(d.feed(0.1780395507812500).is_none());
        assert!(d.feed(0.1803588867187500).is_none());
        assert!(d.feed(0.1817321777343750).is_none());
        assert!(d.feed(0.1816711425781250).is_none());
        assert!(d.feed(0.1799926757812500).is_some());
    }
}
