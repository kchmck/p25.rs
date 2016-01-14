use std;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum P25Error {
    ReedSolomonUnrecoverable,
    BCHUnrecoverable,
    GolayUnrecoverable,
    HammingUnrecoverable,
    CyclicUnrecoverable,
    ViterbiUnrecoverable,
    UnknownNID,
}

pub type Result<T> = std::result::Result<T, P25Error>;
