use super::bits;

/// P25 baud rate is 4800 symbols/sec = 9600 bits/sec.
const BAUD: usize = 4800;

/// Yields a series of scaled impulses vs time corresponding to given dibits.
pub struct C4FMImpulse<T: Iterator<Item = bits::Dibit>> {
    /// The dibit source to iterate over.
    src: T,
    /// Number of samples per impulse/symbol period.
    samples_per_impulse: usize,
    /// Current global sample index.
    sample: usize,
}

impl<T: Iterator<Item = bits::Dibit>> C4FMImpulse<T> {
    /// Construct a new `C4FMImpulse<T>` from the given source and sample rate.
    pub fn new(src: T, sample_rate: usize) -> C4FMImpulse<T> {
        // Fractional samples aren't supported.
        assert!(sample_rate % BAUD == 0);

        C4FMImpulse {
            src: src,
            samples_per_impulse: sample_rate / BAUD,
            sample: 0,
        }
    }
}

impl<T: Iterator<Item = bits::Dibit>> Iterator for C4FMImpulse<T> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        // Store current sample and move to the next.
        let s = self.sample;
        self.sample += 1;

        // Impulse is only output at the beginning of a symbol period.
        if s % self.samples_per_impulse != 0 {
            return Some(0.0);
        }

        // Map the current dibit to a scaled impulse.
        if let Some(dibit) = self.src.next() {
            match dibit {
                bits::Dibit(0b01) => Some(1800.0),
                bits::Dibit(0b00) => Some(600.0),
                bits::Dibit(0b10) => Some(-600.0),
                bits::Dibit(0b11) => Some(-1800.0),
                _ => panic!("invalid dibit encountered"),
            }
        } else {
            None
        }
    }
}

/// Generates the alternating series of dibits used for the C4FM deviation test. The
/// resulting filtered waveform approximates a 1200Hz sine wave.
pub struct C4FMDeviationDibits {
    /// Used to alternate dibits.
    idx: usize,
}

impl C4FMDeviationDibits {
    /// Construct a new `C4FMDeviationDibits`.
    pub fn new() -> C4FMDeviationDibits {
        C4FMDeviationDibits {
            idx: 0,
        }
    }
}

impl Iterator for C4FMDeviationDibits {
    type Item = bits::Dibit;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;

        self.idx += 1;
        self.idx %= 4;

        Some(if idx < 2 {
            bits::Dibit(0b01)
        } else {
            bits::Dibit(0b11)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::bits;

    #[test]
    fn test_impulses() {
        const BITS: &'static [u8] = &[
            0b00011011,
        ];

        let d = bits::iter_dibits(BITS.iter().cloned());
        let mut imp = C4FMImpulse::new(d, 9600);

        assert!(imp.next().unwrap() == 600.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 1800.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == -600.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == -1800.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().is_none());
    }

    #[test]
    fn test_deviation() {
        let mut d = C4FMDeviationDibits::new();

        assert!(d.next().unwrap().0 == 0b01);
        assert!(d.next().unwrap().0 == 0b01);
        assert!(d.next().unwrap().0 == 0b11);
        assert!(d.next().unwrap().0 == 0b11);
        assert!(d.next().unwrap().0 == 0b01);
        assert!(d.next().unwrap().0 == 0b01);
        assert!(d.next().unwrap().0 == 0b11);
        assert!(d.next().unwrap().0 == 0b11);
    }
}
