//! Encoding and decoding for the several error correction coding schemes used in P25.

#[macro_use]
mod macros;

pub mod bch;
pub mod bmcf;
pub mod cyclic;
pub mod galois;
pub mod golay;
pub mod hamming;
pub mod reed_solomon;
pub mod trellis;
