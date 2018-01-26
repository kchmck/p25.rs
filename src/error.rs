//! Standard errors that may occur when working with P25.

use std;

/// P25 runtime errors.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum P25Error {
    /// Too many errors were detected when attempting an RS-short decode.
    RsShortUnrecoverable,
    /// Too many errors were detected when attempting an RS-medium decode.
    RsMediumUnrecoverable,
    /// Too many errors were detected when attempting an RS-long decode.
    RsLongUnrecoverable,
    /// Too many errors were detected when attempting a BCH decode.
    BchUnrecoverable,
    /// Too many errors were detected when attempting a standard Golay decode.
    GolayStdUnrecoverable,
    /// Too many errors were detected when attempting a shortened Golay decode.
    GolayShortUnrecoverable,
    /// Too many errors were detected when attempting an extended Golay decode.
    GolayExtUnrecoverable,
    /// Too many errors were detected when attempting a standard Hamming decode.
    HammingStdUnrecoverable,
    /// Too many errors were detected when attempting a shortened Hamming decode.
    HammingShortUnrecoverable,
    /// Too many errors were detected when attempting a cyclic decode.
    CyclicUnrecoverable,
    /// An ambiguous symbol or too many errors were detected when attempting to decode the
    /// dibit Viterbi code.
    DibitViterbiUnrecoverable,
    /// An unknown or corrupted NID was encountered.
    UnknownNid,
}

/// Standard result using `P25Error`.
pub type Result<T> = std::result::Result<T, P25Error>;
