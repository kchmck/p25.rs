//! Runtime statistics.

/// Records various runtime statistics.
#[derive(Copy, Clone)]
pub struct Stats {
    bch_err: usize,
    bch_total: usize,

    cyclic_err: usize,
    cyclic_total: usize,

    golay_std_err: usize,
    golay_std_total: usize,

    golay_ext_err: usize,
    golay_ext_total: usize,

    golay_short_err: usize,
    golay_short_total: usize,

    hamming_std_err: usize,
    hamming_std_total: usize,

    hamming_short_err: usize,
    hamming_short_total: usize,

    rs_short_err: usize,
    rs_short_total: usize,

    rs_med_err: usize,
    rs_med_total: usize,

    rs_long_err: usize,
    rs_long_total: usize,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            bch_err: 0,
            bch_total: 0,

            cyclic_err: 0,
            cyclic_total: 0,

            golay_std_err: 0,
            golay_std_total: 0,

            golay_ext_err: 0,
            golay_ext_total: 0,

            golay_short_err: 0,
            golay_short_total: 0,

            hamming_std_err: 0,
            hamming_std_total: 0,

            hamming_short_err: 0,
            hamming_short_total: 0,

            rs_short_err: 0,
            rs_short_total: 0,

            rs_med_err: 0,
            rs_med_total: 0,

            rs_long_err: 0,
            rs_long_total: 0,
        }
    }
}

impl Stats {
    /// Merge in the stats from the given object and reset the other stats back to
    /// default.
    pub fn merge<T: HasStats>(&mut self, other: &mut T) {
        let stats = other.stats();

        self.bch_err += stats.bch_err;
        self.bch_total += stats.bch_total;

        self.cyclic_err += stats.cyclic_err;
        self.cyclic_total += stats.cyclic_total;

        self.golay_std_err += stats.golay_std_err;
        self.golay_std_total += stats.golay_std_total;

        self.golay_ext_err += stats.golay_ext_err;
        self.golay_ext_total += stats.golay_ext_total;

        self.golay_short_err += stats.golay_short_err;
        self.golay_short_total += stats.golay_short_total;

        self.hamming_std_err += stats.hamming_std_err;
        self.hamming_std_total += stats.hamming_std_total;

        self.hamming_short_err += stats.hamming_short_err;
        self.hamming_short_total += stats.hamming_short_total;

        self.rs_short_err += stats.rs_short_err;
        self.rs_short_total += stats.rs_short_total;

        self.rs_med_err += stats.rs_med_err;
        self.rs_med_total += stats.rs_med_total;

        self.rs_long_err += stats.rs_long_err;
        self.rs_long_total += stats.rs_long_total;

        stats.clear();
    }

    /// Clear all stats.
    pub fn clear(&mut self) {
        *self = Stats::default();
    }

    /// Record BCH errors.
    pub fn record_bch(&mut self, err: usize) {
        self.bch_err += err;
        self.bch_total += 64;
    }

    /// Record cyclic code errors.
    pub fn record_cyclic(&mut self, err: usize) {
        self.cyclic_err += err;
        self.cyclic_total += 16;
    }

    /// Record standard Golay errors.
    pub fn record_golay_std(&mut self, err: usize) {
        self.golay_std_err += err;
        self.golay_std_total += 23;
    }

    /// Record extended Golay errors.
    pub fn record_golay_ext(&mut self, err: usize) {
        self.golay_ext_err += err;
        self.golay_ext_total += 24;
    }

    /// Record short Golay errors.
    pub fn record_golay_short(&mut self, err: usize) {
        self.golay_short_err += err;
        self.golay_short_total += 18;
    }

    /// Record standard Hamming code errors.
    pub fn record_hamming_std(&mut self, err: usize) {
        self.hamming_std_err += err;
        self.hamming_std_total += 15;
    }

    /// Record short Hamming code errors.
    pub fn record_hamming_short(&mut self, err: usize) {
        self.hamming_short_err += err;
        self.hamming_short_total += 10;
    }

    /// Record short Reed-Solomon code errors.
    pub fn record_rs_short(&mut self, err: usize) {
        self.rs_short_err += err;
        self.rs_short_total += 24;
    }

    /// Record medium Reed-Solomon code errors.
    pub fn record_rs_med(&mut self, err: usize) {
        self.rs_med_err += err;
        self.rs_med_total += 24;
    }

    /// Record long Reed-Solomon code errors.
    pub fn record_rs_long(&mut self, err: usize) {
        self.rs_long_err += err;
        self.rs_long_total += 36;
    }
}

/// Indicates that a type captures statistics.
pub trait HasStats {
    /// Retrieve captured statistics.
    fn stats(&mut self) -> &mut Stats;
}
