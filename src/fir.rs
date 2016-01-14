//! Defines the `FIRFilter` structure for FIR filtering.

use std::cmp;

/// A FIR filter for convolving with a series of samples.
pub struct FIRFilter<'a> {
    /// The filter coefficients for multiplying with the input signal, represents bi.
    coefs: &'a [f32],
    /// A ring buffer of samples in the signal, represents x[i].
    history: Vec<f32>,
    /// The index of the most-recently added sample, represents n in x[n].
    idx: usize,
}

impl<'a> FIRFilter<'a> {
    /// Construct an order-N filter with the given N+1 coefficients.
    pub fn new(coefs: &'a [f32]) -> FIRFilter<'a> {
        FIRFilter {
            coefs: coefs,
            history: vec![0.0; coefs.len()],
            idx: 0,
        }
    }

    /// Perform the convolution with the current history of samples. Calculates
    /// y[n] = c0*x[n] + c1*x[n-1] + cN*x[n-N].
    fn calc(&self) -> f32 {
        // Copy the current index so we can move backwards.
        let mut cur = self.idx;

        self.coefs.iter().fold(0.0, |s, &coef| {
            // Wrap around to the last sample after visiting the first.
            cur = cmp::min(cur - 1, self.history.len() - 1);
            // Accumulate the next term.
            s + coef * self.history[cur]
        })
    }

    /// Add a sample to the current history and calculate the convolution.
    pub fn feed(&mut self, sample: f32) -> f32 {
        // Store the given sample in the current history slot.
        self.history[self.idx] = sample;

        // Move to the next slot and wrap around.
        self.idx += 1;
        self.idx %= self.history.len();

        self.calc()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_fir() {
        use super::*;

        const COEFS: &'static [f32] = &[
            0.0,
            1.0,
            0.0,
            1.0,
        ];

        let mut f = FIRFilter::new(COEFS);

        assert!(f.feed(100.0) == 0.0);
        assert!(f.feed(200.0) == 100.0);
        assert!(f.feed(300.0) == 200.0);
        assert!(f.feed(400.0) == 400.0);
        assert!(f.feed(0.0) == 600.0);
        assert!(f.feed(0.0) == 300.0);
        assert!(f.feed(0.0) == 400.0);
        assert!(f.feed(0.0) == 0.0);
        assert!(f.feed(0.0) == 0.0);
    }
}
