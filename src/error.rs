//! Standard errors that may occur when working with P25.

use std;

/// P25 runtime errors.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum P25Error {
    /// Too many errors were detected when attempting a RS decode.
    ReedSolomonUnrecoverable,
    /// Too many errors were detected when attempting a BCH decode.
    BchUnrecoverable,
    /// Too many errors were detected when attempting a Golay decode.
    GolayUnrecoverable,
    /// Too many errors were detected when attempting a Hamming decode.
    HammingUnrecoverable,
    /// Too many errors were detected when attempting a cyclic decode.
    CyclicUnrecoverable,
    /// An ambiguous symbol or too many errors were detected when attempting convolutional decode.
    ViterbiUnrecoverable,
    /// An unknown or corrupted NID was encountered.
    UnknownNID,
}

/// Standard result using `P25Error`.
pub type Result<T> = std::result::Result<T, P25Error>;
