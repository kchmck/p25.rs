use bits;
use consts;

const DECIDER_HEADROOM: f32 = 0.70;

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
    energy: f32,
}

impl Correlator {
    pub fn new() -> Correlator {
        Correlator {
            pos: 0,
            energy: 0.0,
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
            Some(self.energy)
        } else {
            None
        }
    }

    fn add(&mut self, s: f32) {
        const MATCHED_FILTER: &'static [f32] = &[
            0.0,
            0.0,
            0.0,
            0.0,
            0.9827855224082289,
            1.0,
            0.9827855224082289,
            0.0,
            0.0,
            0.0,
            0.0,
        ];

        if MATCHED_FILTER[self.pos] != 0.0 {
            println!("{}", s);
        }

        self.energy += s * MATCHED_FILTER[self.pos];
        self.pos += 1;
    }
}

#[derive(Copy, Clone)]
pub struct Decider {
    high_thresh: f32,
}

impl Decider {
    pub fn new(high_thresh: f32) -> Decider {
        Decider {
            high_thresh: high_thresh * DECIDER_HEADROOM,
        }
    }

    pub fn decide(&self, energy: f32) -> bits::Dibit {
        // println!("decide {} {}", energy, self.high_thresh);

        if energy >= self.high_thresh {
            bits::Dibit::new(0b01)
        } else if energy >= 0.0 {
            bits::Dibit::new(0b00)
        } else if energy <= -self.high_thresh {
            bits::Dibit::new(0b11)
        } else {
            bits::Dibit::new(0b10)
        }
    }
}
