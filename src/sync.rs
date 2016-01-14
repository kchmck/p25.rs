use std;

use baseband::{DCOffsetCorrector, Decider, Correlator, Decoder};
use consts;

use self::PeakType::*;
use self::SyncError::*;
use self::SyncState::*;

pub enum SyncError {
    InvalidRun,
    InvalidSine,
}

enum SyncState {
    BootstrapRun(RunCheck),
    BigSine(Peaks),
    MidRun(RunCheck),
    SmallSine(Peaks),
    LockBoundary(SymbolClock),
    EndRun(DCOffsetCorrector, Correlator),
    Locked(Decoder),
    Error(SyncError),
}

pub struct SyncDetector {
    state: SyncState,
    timing: Timing,
    sums: Sums,
    dco: DCOffset,
}

impl SyncDetector {
    pub fn new() -> SyncDetector {
        SyncDetector {
            state: BootstrapRun(RunCheck::new(4 * consts::PERIOD, None)),
            timing: Timing::new(),
            sums: Sums::new(),
            dco: DCOffset::new(),
        }
    }

    /// Take the given sample and sample time and output where the state machine should
    /// move next.
    fn handle(&mut self, s: f32, t: usize) -> Option<SyncState> {
        match self.state {
            BootstrapRun(ref mut run) => match run.feed(s) {
                Some(true) => Some(BigSine(Peaks::new(Maximum, s))),
                Some(false) => Some(Error(InvalidRun)),
                None => None,
            },
            BigSine(ref mut peaks) => match peaks.feed(s) {
                Some(Maximum) if self.timing.pos > 4 || self.timing.pos % 2 == 0 =>
                    Some(Error(InvalidSine)),
                Some(Minimum) if self.timing.pos > 3 || self.timing.pos % 2 != 0 =>
                    Some(Error(InvalidSine)),
                Some(m) => {
                    self.timing.add(t - 1);

                    match m {
                        Maximum if self.timing.pos == 4 => Some(MidRun(
                            RunCheck::new(3 * consts::PERIOD, Some(consts::PERIOD))
                        )),
                        _ => None,
                    }
                },
                None => None,
            },
            MidRun(ref mut run) => match run.feed(-s) {
                Some(true) => Some(SmallSine(Peaks::new(Minimum, s))),
                Some(false) => Some(Error(InvalidRun)),
                None => None,
            },
            SmallSine(ref mut peaks) => match peaks.feed(s) {
                Some(Maximum) if self.timing.pos > 7 || self.timing.pos % 2 != 0 =>
                    Some(Error(InvalidSine)),
                Some(Minimum) if self.timing.pos > 6 || self.timing.pos % 2 == 0 =>
                    Some(Error(InvalidSine)),
                Some(m) => {
                    self.dco.add(prev);
                    self.timing.add(t - 1);

                    match m {
                        Maximum if self.timing.pos == 7 => Some(LockBoundary(
                            SymbolClock::new(self.timing.corrected_start())
                        )),
                        _ => None,
                    }
                },
                None => None,
            },
            LockBoundary(ref mut clock) => if clock.boundary(t + 1) {
                Some(EndRun(
                    DCOffsetCorrector::new(self.dco.correction()),
                    Correlator::new()
                ))
            } else {
                None
            },
            EndRun(ref dco, ref mut corr) => match corr.feed(dco.feed(s)) {
                Some(sum) if sum > 0.0 => Some(Error(InvalidRun)),
                Some(sum) => {
                    if self.sums.add(sum) {
                        Some(Locked(
                            Decoder::new(*dco, Correlator::primed(s),
                                         Decider::new(self.sums.min()))
                        ))
                    } else {
                        Some(EndRun(*dco, Correlator::primed(s)))
                    }
                },
                None => None,
            },
            Error(_) | Locked(_) => panic!(),
        }
    }

    pub fn feed(&mut self, s: f32, t: usize) -> Option<Result<Decoder, SyncError>> {
        match self.handle(s, t) {
            Some(Error(e)) => Some(Err(e)),
            Some(Locked(d)) => Some(Ok(d)),
            Some(next) => {
                self.state = next;
                None
            },
            None => None,
        }
    }
}

/// Recovers impulse timing from the (positive and negative) peaks of the "big sine" and
/// "small sine" sections of the frame sync waveform.
struct Timing {
    /// Times of the peaks in the waveform.
    times: [usize; 7],
    /// Current number of peaks (length of `times`.)
    pub pos: usize,
}

impl Timing {
    /// Construct a new `Timing` with the given symbol period.
    pub fn new() -> Timing {
        Timing {
            times: [0; 7],
            pos: 0,
        }
    }

    /// Add a new peak time.
    pub fn add(&mut self, t: usize) {
        self.times[self.pos] = t;
        self.pos += 1;
    }

    /// Expand the peak times into impulse times (the big sine peaks are made by two
    /// impulses.)
    fn expand(&self) -> [usize; 10] {
        let half_period = consts::PERIOD / 2;

        [
            self.times[0],
            self.times[1] - half_period,
            self.times[1] + half_period,
            self.times[2] - half_period,
            self.times[2] + half_period,
            self.times[3] - half_period,
            self.times[3] + half_period,
            self.times[4],
            self.times[5],
            self.times[6],
        ]
    }

    /// Get the uncorrected starting time of the impulse clock.
    fn start(&self) -> usize { self.times[0] }

    /// Calculate the timing correction to apply to the impulse clock.
    fn correction(&self) -> f32 {
        // These are the raw expected impulse times relative to the first impulse.
        const EXPECTED_TIMES: &'static [usize] = &[
            0, 1, 2, 3, 4, 5, 6, 11, 12, 13
        ];

        assert!(self.pos == 7);

        // Scale the impulse times by the symbol period.
        let expected = EXPECTED_TIMES.iter().map(|e| e * consts::PERIOD);
        let expanded = self.expand();

        // Calculate the average difference between real and expected timings.
        expanded.iter().map(|t| {
            // Calculate the times relative to the first impulse.
            t - self.start()
        }).zip(expected).map(|(diff, e)| {
            // Calculate the difference from the expected time.
            diff as isize - e as isize
        }).fold(0, |s, d| s + d) as f32 / expanded.len() as f32
    }

    /// Get the corrected impulse clock starting time.
    pub fn corrected_start(&self) -> usize {
        (self.start() as isize + self.correction().round() as isize) as usize
    }
}

/// Calculates symbol impulse and boundary times.
struct SymbolClock {
    /// Impulse starting time.
    start: usize,
}

impl SymbolClock {
    /// Construct a new `SymbolClock` with the given impulse clock starting time and
    /// symbol period.
    pub fn new(start: usize) -> SymbolClock {
        SymbolClock {
            start: start,
        }
    }

    /// Check if the given time falls on a symbol impulse.
    pub fn impulse(&self, t: usize) -> bool {
        (t - self.start) % consts::PERIOD == 0
    }

    /// Check if the given time falls on a symbol boundary.
    pub fn boundary(&self, t: usize) -> bool {
        self.impulse(t + consts::PERIOD / 2)
    }
}

#[derive(Copy, Clone)]
/// A maximum or minimum peak.
enum PeakType {
    Maximum,
    Minimum,
}

#[derive(Copy, Clone)]
/// Finds peaks in a waveform.
struct Peaks {
    /// Previous inflection found.
    state: PeakType,
    /// Value of the previous sample.
    prev: f32,
}

impl Peaks {
    /// Constructs a new `Peaks` with the given starting state and sample.
    pub fn new(state: PeakType, start: f32) -> Peaks {
        Peaks {
            state: state,
            prev: start,
        }
    }

    /// Feed in a sample and check for an inflection. Return `Some(m, p)`, where `m` is
    /// the inflection type and `p` is the value of the previous sample, if the previous
    /// sample was at a peak, and return `None` otherwise.
    pub fn feed(&mut self, s: f32) -> Option<PeakType> {
        let prev = self.prev;
        self.prev = s;

        match self.cmp(prev, s) {
            Some(st) => {
                self.state = st;
                Some(st)
            },
            None => None,
        }
    }

    /// Compare the current and previous samples and check if the previous was at an
    /// inflection. Return `Some(m)` with the inflection type `m` if so and `None`
    /// otherwise.
    fn cmp(&self, prev: f32, cur: f32) -> Option<PeakType> {
        match self.state {
            // If we're coming off a maximum, the slope should be headed downwards.
            Maximum if cur <= prev => None,
            // Since cur < prev, the previous sample was at a minimum.
            Maximum => Some(Minimum),

            // If we're coming off a minimum, the slope should be headed upwards.
            Minimum if cur >= prev => None,
            // Since cur > prev, the previous sample was at a maximum.
            Minimum => Some(Maximum),
        }
    }
}

/// Checks for a "run" of impulses of a certain length.
/// - positive run only
/// - sample skipping
struct RunCheck {
    /// Required length of the run.
    length: usize,
    /// (Optional) initial samples remaining that may be skipped.
    skip_remain: Option<usize>,
    /// Current length of the run.
    run: usize,
}

impl RunCheck {
    /// Construct a new `RunCheck` with the given required run length and maximum amount
    /// of initia samples to skip (can be `None` to disable skipping.)
    pub fn new(length: usize, max_skip: Option<usize>) -> RunCheck {
        RunCheck {
            length: length,
            skip_remain: max_skip,
            run: 0,
        }
    }

    /// Feed the given sample into the current state and return `Some(true)` if it
    /// completes the run, `Some(false)` if the run wasn't the required length, and `None`
    /// if more samples must be fed in.
    pub fn feed(&mut self, s: f32) -> Option<bool> {
        if s > 0.0 {
            self.run += 1;
            None
        } else if self.run == 0 {
            match self.skip_remain {
                Some(0) => Some(false),
                Some(ref mut remain) => {
                    *remain -= 1;
                    None
                },
                None => None,
            }
        } else if self.run < self.length {
            Some(false)
        } else {
            Some(true)
        }
    }
}

struct Sums {
    sums: [f32; 5],
    pos: usize,
}

impl Sums {
    pub fn new() -> Sums {
        Sums {
            sums: [0.0; 5],
            pos: 0,
        }
    }

    pub fn add(&mut self, sum: f32) -> bool {
        self.sums[self.pos] = sum.abs();
        self.pos += 1;
        self.pos == 5
    }

    pub fn min(&self) -> f32 {
        assert!(self.pos == 5);

        self.sums.iter().fold(std::f32::MAX, |s, &x| {
            match s.partial_cmp(&x).unwrap() {
                std::cmp::Ordering::Less | std::cmp::Ordering::Equal => s,
                std::cmp::Ordering::Greater => x,
            }
        })
    }
}

struct DCOffset {
    peaks: [f32; 3],
    pos: usize,
}

impl DCOffset {
    pub fn new() -> DCOffset {
        DCOffset {
            peaks: [0.0; 3],
            pos: 0,
        }
    }

    pub fn reset(&mut self) {
        self.pos = 0;
    }

    pub fn add(&mut self, s: f32) {
        self.peaks[self.pos] = s;
        self.pos += 1;
    }

    fn min(&self) -> f32 { self.peaks[1] }
    fn max(&self) -> f32 { (self.peaks[0] + self.peaks[2]) / 2.0 }

    fn delta(&self) -> f32 {
        const EXPECTED_DIFF: f32 = 0.032776727;

        let min = self.min();
        let max = self.max();

        (max - min) * EXPECTED_DIFF - (max + min)
    }

    pub fn correction(&self) -> f32 {
        self.delta() / 2.0
    }
}

#[cfg(test)]
mod test {
    use super::{Timing, SymbolClock, Peaks, RunCheck, Sums, DCOffset};
    use super::PeakType::*;

    #[test]
    fn test_timing_perfect() {
        let mut t = Timing::new();
        t.add(17);
        t.add(32);
        t.add(52);
        t.add(72);
        t.add(127);
        t.add(137);
        t.add(147);

        let e = t.expand();
        assert_eq!(e[0], 17);
        assert_eq!(e[1], 27);
        assert_eq!(e[2], 37);
        assert_eq!(e[3], 47);
        assert_eq!(e[4], 57);
        assert_eq!(e[5], 67);
        assert_eq!(e[6], 77);
        assert_eq!(e[7], 127);
        assert_eq!(e[8], 137);
        assert_eq!(e[9], 147);

        assert_eq!(t.start(), 17);
        assert_eq!(t.correction(), 0.0);
        assert_eq!(t.corrected_start(), 17);
    }

    #[test]
    fn test_timing_jitter() {
        let mut t = Timing::new();
        t.add(17);
        t.add(31);
        t.add(51);
        t.add(71);
        t.add(126);
        t.add(136);
        t.add(146);

        assert_eq!(t.correction().round(), -1.0);
        assert_eq!(t.corrected_start(), 16);
    }

    #[test]
    fn test_timing_mixed() {
        let mut t = Timing::new();
        t.add(17);
        t.add(32);
        t.add(52);
        t.add(72);
        t.add(125);
        t.add(139);
        t.add(147);

        assert_eq!(t.correction().round(), 0.0);
        assert_eq!(t.corrected_start(), 17);
    }

    #[test]
    fn test_clock() {
        let s = SymbolClock::new(12);
        assert!(s.impulse(12));
        assert!(s.boundary(17));
        assert!(s.impulse(22));
        assert!(s.boundary(27));
        assert!(s.impulse(32));
        assert!(s.boundary(37));
    }

    #[test]
    fn test_run_lenient() {
        let mut run = RunCheck::new(3, Some(3));
        assert!(if let None = run.feed(-2.0) { true } else { false });
        assert!(if let None = run.feed(-1.0) { true } else { false });
        assert!(if let None = run.feed(0.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(2.0) { true } else { false });
        assert!(if let Some(true) = run.feed(-1.0) { true } else { false });
    }

    #[test]
    fn test_run_skip() {
        let mut run = RunCheck::new(3, Some(3));
        assert!(if let None = run.feed(-2.0) { true } else { false });
        assert!(if let None = run.feed(-1.0) { true } else { false });
        assert!(if let None = run.feed(0.0) { true } else { false });
        assert!(if let Some(false) = run.feed(0.0) { true } else { false });
    }

    #[test]
    fn test_run_detect() {
        let mut run = RunCheck::new(3, None);
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(2.0) { true } else { false });
        assert!(if let Some(true) = run.feed(-3.0) { true } else { false });
    }

    #[test]
    fn test_run_interrupt() {
        let mut run = RunCheck::new(3, None);
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let Some(false) = run.feed(-3.0) { true } else { false });
    }

    #[test]
    fn test_run_inverse() {
        let mut run = RunCheck::new(3, None);
        assert!(if let None = run.feed(-1.0) { true } else { false });
        assert!(if let None = run.feed(-0.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(1.0) { true } else { false });
        assert!(if let None = run.feed(2.0) { true } else { false });
        assert!(if let Some(true) = run.feed(-3.0) { true } else { false });
    }

    #[test]
    fn test_inflection() {
        let mut p = Peaks::new(Maximum, 0.0);
        assert!(if let None = p.feed(0.0) { true } else { false });
        assert!(if let None = p.feed(-1.0) { true } else { false });
        assert!(if let None = p.feed(-2.0) { true } else { false });
        assert!(if let Some(Minimum) = p.feed(-1.0) { true } else { false });
        assert!(if let None = p.feed(-1.0) { true } else { false });
        assert!(if let None = p.feed(1.0) { true } else { false });
        assert!(if let None = p.feed(2.0) { true } else { false });
        assert!(if let Some(Maximum) = p.feed(1.0) { true } else { false });
        assert!(if let None = p.feed(0.0) { true } else { false });
    }

    #[test]
    fn test_sums() {
        let mut s = Sums::new();
        s.add(0.0);
        s.add(1.0);
        s.add(2.0);
        s.add(31.0);
        s.add(1.0);
        assert!(s.min() == 0.0);
    }

    #[test]
    fn test_dc_offset() {
        let mut dc = DCOffset::new();

        dc.add(6645.9780238467865);
        dc.add(-6175.7425637272545);
        dc.add(6542.627695382622);

        assert!(dc.correction().abs() < 0.001);
    }
}
