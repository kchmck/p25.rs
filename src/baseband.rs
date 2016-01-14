use std;

use bits;
use consts;

#[derive(Copy, Clone)]
pub struct Decoder {
    corrector: DCOffsetCorrector,
    correlator: Correlator,
    decider: Decider,
}

impl Decoder {
    pub fn new(corrector: DCOffsetCorrector, correlator: Correlator, decider: Decider)
        -> Decoder
    {
        Decoder {
            corrector: corrector,
            correlator: correlator,
            decider: decider,
        }
    }

    fn reset(&mut self, s: f32) {
        self.correlator = Correlator::primed(s);
    }

    pub fn feed(&mut self, s: f32) -> Option<bits::Dibit> {
        match self.correlator.feed(self.corrector.feed(s)) {
            Some(sum) => {
                self.reset(s);
                Some(self.decider.decide(sum))
            },
            None => None,
        }
    }
}

#[derive(Copy, Clone)]
/// Simply corrects the DC offset in a waveform.
pub struct DCOffsetCorrector {
    /// Offset to add to each sample.
    correction: f32,
}

impl DCOffsetCorrector {
    pub fn new(correction: f32) -> DCOffsetCorrector {
        DCOffsetCorrector {
            correction: correction,
        }
    }

    pub fn feed(&self, s: f32) -> f32 {
        s + self.correction
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
