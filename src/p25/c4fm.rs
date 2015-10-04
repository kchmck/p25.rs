use std;

use bits;
use system::{SystemParams, P25Params};

/// Yields a series of scaled impulses vs time corresponding to given dibits.
pub struct C4FMImpulse<T, S: SystemParams> {
    system: std::marker::PhantomData<S>,
    /// The dibit source to iterate over.
    src: T,
    /// Current global sample index.
    sample: usize,
}

impl<T, S = P25Params> C4FMImpulse<T, S> where
    T: Iterator<Item = bits::Dibit>,
    S: SystemParams
{
    /// Construct a new `C4FMImpulse<T>` from the given source and sample rate.
    pub fn new(src: T) -> C4FMImpulse<T, S> {
        C4FMImpulse {
            system: std::marker::PhantomData,
            src: src,
            sample: 0,
        }
    }
}

impl<T, S> Iterator for C4FMImpulse<T, S> where
    T: Iterator<Item = bits::Dibit>,
    S: SystemParams
{
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        // Store current sample and move to the next.
        let s = self.sample;
        self.sample += 1;

        // Impulse is only output at the beginning of a symbol period.
        if s % S::period() != 0 {
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

        let d = bits::Dibits::new(BITS.iter().cloned());
        let mut imp = C4FMImpulse::new(d);

        assert!(imp.next().unwrap() == 600.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 1800.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == -600.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == -1800.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
        assert!(imp.next().unwrap() == 0.0);
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
