use std;

use bits;
use consts;

#[derive(Copy, Clone)]
pub struct Decoder {
    correlator: Correlator,
    decider: Decider,
}

impl Decoder {
    pub fn new(correlator: Correlator, decider: Decider) -> Decoder {
        Decoder {
            correlator: correlator,
            decider: decider,
        }
    }

    fn reset(&mut self, s: f32) {
        self.correlator = Correlator::primed(s);
    }

    pub fn feed(&mut self, s: f32) -> Option<bits::Dibit> {
        match self.correlator.feed(s) {
            Some(sum) => {
                self.reset(s);
                Some(self.decider.decide(sum))
            },
            None => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Correlator {
    pos: usize,
    sum: f32,
}

impl Correlator {
    pub fn new() -> Correlator {
        Correlator {
            pos: 0,
            sum: 0.0,
        }
    }

    pub fn primed(s: f32) -> Correlator {
        let mut c = Correlator::new();
        c.add(s);
        c
    }

    pub fn feed(&mut self, s: f32) -> Option<f32> {
        self.add(s);

        if self.pos > consts::PERIOD {
            Some(self.sum)
        } else {
            None
        }
    }

    fn add(&mut self, s: f32) {
        const MATCHED_FILTER: &'static [f32] = &[
            0.6290605212918821,
            0.7507772559612889,
            0.8542215065015759,
            0.933168001531859,
            0.9827855224082289,
            1.0,
            0.9827855224082289,
            0.933168001531859,
            0.8542215065015759,
            0.7507772559612889,
            0.6290605212918821,
        ];

        self.sum += s * MATCHED_FILTER[self.pos];
        self.pos += 1;
    }
}

#[derive(Copy, Clone)]
pub struct Decider {
    high_thresh: f32,
}

impl Decider {
    pub fn new(high_thresh: f32) -> Decider {
        const FUDGE: f32 = 0.75;

        Decider {
            high_thresh: high_thresh * FUDGE,
        }
    }

    pub fn decide(&self, sum: f32) -> bits::Dibit {
        if sum >= self.high_thresh {
            bits::Dibit::new(0b01)
        } else if sum >= 0.0 {
            bits::Dibit::new(0b00)
        } else if sum <= -self.high_thresh {
            bits::Dibit::new(0b11)
        } else {
            bits::Dibit::new(0b10)
        }
    }
}
