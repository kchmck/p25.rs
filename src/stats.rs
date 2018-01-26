//! Runtime statistics.

use error::P25Error;

/// Tracks stats for an error correction code.
#[derive(Copy, Clone)]
pub struct CodeStats {
    /// Number of symbols per word.
    ///
    /// TODO: make this a const-generic or similar.
    size: usize,
    /// Total number of received words.
    words: usize,
    /// Number of corrected symbols.
    fixed: usize,
    /// Number of unrecoverable words.
    err: usize,
}

impl CodeStats {
    /// Create a new `CodeStats` with empty counters for the code with the given number of
    /// symbols per word.
    fn new(size: usize) -> Self {
        CodeStats {
            size: size,
            words: 0,
            err: 0,
            fixed: 0,
        }
    }

    /// Record that a word was received with the given amount of corrected symbols.
    pub fn record_fixes(&mut self, err: usize) {
        debug_assert!(err <= self.size);

        self.words += 1;
        self.fixed += err;
    }

    /// Record that a word was received with an unrecoverable error.
    pub fn record_err(&mut self) {
        self.words += 1;
        self.err += 1;
    }

    /// Merge in the stats from the given object and clear the other stats.
    fn merge(&mut self, other: &mut CodeStats) {
        debug_assert!(self.size == other.size);

        self.words += other.words;
        self.err += other.err;
        self.fixed += other.fixed;

        other.clear();
    }

    /// Clear all stats.
    fn clear(&mut self) {
        self.words = 0;
        self.err = 0;
        self.fixed = 0;
    }
}

/// Records various runtime statistics.
#[derive(Copy, Clone)]
pub struct Stats {
    /// Stats for the BCH code.
    pub bch: CodeStats,
    /// Stats for the cyclic code.
    pub cyclic: CodeStats,
    /// Stats for the standard Golay code.
    pub golay_std: CodeStats,
    /// Stats for the extended Golay code.
    pub golay_ext: CodeStats,
    /// Stats for the short Golay code.
    pub golay_short: CodeStats,
    /// Stats for the standard Hamming code.
    pub hamming_std: CodeStats,
    /// Stats for the short Hamming code.
    pub hamming_short: CodeStats,
    /// Stats for the short RS code.
    pub rs_short: CodeStats,
    /// Stats for the medium RS code.
    pub rs_med: CodeStats,
    /// Stats for the long RS code.
    pub rs_long: CodeStats,
    /// Stats for the dibit Viterbi code.
    pub viterbi_dibit: CodeStats,
    /// Stats for the tribit Viterbi code.
    pub viterbi_tribit: CodeStats,
}

impl Default for Stats {
    fn default() -> Self {
        Stats {
            bch: CodeStats::new(64),
            cyclic: CodeStats::new(16),
            golay_std: CodeStats::new(23),
            golay_ext: CodeStats::new(24),
            golay_short: CodeStats::new(18),
            hamming_std: CodeStats::new(15),
            hamming_short: CodeStats::new(10),
            rs_short: CodeStats::new(24),
            rs_med: CodeStats::new(24),
            rs_long: CodeStats::new(36),
            viterbi_dibit: CodeStats::new(196),
            viterbi_tribit: CodeStats::new(196),
        }
    }
}

impl Stats {
    /// Merge in the stats from the given object and reset the other stats back to
    /// default.
    pub fn merge<T: HasStats>(&mut self, other: &mut T) {
        let stats = other.stats();

        self.bch.merge(&mut stats.bch);
        self.cyclic.merge(&mut stats.cyclic);
        self.golay_std.merge(&mut stats.golay_std);
        self.golay_ext.merge(&mut stats.golay_ext);
        self.golay_short.merge(&mut stats.golay_short);
        self.hamming_std.merge(&mut stats.hamming_std);
        self.hamming_short.merge(&mut stats.hamming_short);
        self.rs_short.merge(&mut stats.rs_short);
        self.rs_med.merge(&mut stats.rs_med);
        self.rs_long.merge(&mut stats.rs_long);
        self.viterbi_dibit.merge(&mut stats.viterbi_dibit);
        self.viterbi_tribit.merge(&mut stats.viterbi_tribit);
    }

    /// Clear all stats.
    pub fn clear(&mut self) {
        *self = Stats::default();
    }

    /// Record the given error into the current stats.
    pub fn record_err(&mut self, err: P25Error) {
        use error::P25Error::*;

        match err {
            BchUnrecoverable => self.bch.record_err(),
            CyclicUnrecoverable => self.cyclic.record_err(),
            GolayStdUnrecoverable => self.golay_std.record_err(),
            GolayExtUnrecoverable => self.golay_ext.record_err(),
            GolayShortUnrecoverable => self.golay_short.record_err(),
            HammingStdUnrecoverable => self.hamming_std.record_err(),
            HammingShortUnrecoverable => self.hamming_short.record_err(),
            RsShortUnrecoverable => self.rs_short.record_err(),
            RsMediumUnrecoverable => self.rs_med.record_err(),
            RsLongUnrecoverable => self.rs_long.record_err(),
            DibitViterbiUnrecoverable => self.viterbi_dibit.record_err(),
            UnknownNid => {},
        }
    }
}

/// Indicates that a type captures statistics.
pub trait HasStats {
    /// Retrieve captured statistics.
    fn stats(&mut self) -> &mut Stats;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_code_stats() {
        let mut a = CodeStats::new(23);
        let mut b = CodeStats::new(23);

        a.record_fixes(13);
        a.record_err();
        assert_eq!(a.size, 23);
        assert_eq!(a.words, 2);
        assert_eq!(a.fixed, 13);
        assert_eq!(a.err, 1);

        b.record_fixes(11);
        b.record_fixes(19);
        b.record_err();
        b.record_err();
        assert_eq!(b.size, 23);
        assert_eq!(b.words, 4);
        assert_eq!(b.fixed, 30);
        assert_eq!(b.err, 2);

        a.merge(&mut b);
        assert_eq!(a.size, 23);
        assert_eq!(a.words, 6);
        assert_eq!(a.fixed, 43);
        assert_eq!(a.err, 3);
        assert_eq!(b.size, 23);
        assert_eq!(b.words, 0);
        assert_eq!(b.fixed, 0);
        assert_eq!(b.err, 0);

        let mut c = CodeStats::new(7);
        c.record_fixes(3);
        c.record_fixes(2);
        c.record_err();
        assert_eq!(c.size, 7);
        assert_eq!(c.words, 3);
        assert_eq!(c.fixed, 5);
        assert_eq!(c.err, 1);
    }
}
