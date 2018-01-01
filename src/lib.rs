//! Implements the Project 25 (P25) air interface radio protocol, including baseband frame
//! synchronization, symbol decoding, error correction coding, and packet reconstuction.

#![feature(const_fn)]
#![feature(inclusive_range_syntax)]

extern crate binfield_matrix;
extern crate cai_cyclic;
extern crate collect_slice;
extern crate moving_avg;
extern crate num;

#[cfg(feature = "ser")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "ser")]
extern crate serde;

#[macro_use]
extern crate static_fir;

mod buffer;
mod util;

pub mod baseband;
pub mod bits;
pub mod coding;
pub mod consts;
pub mod data;
pub mod error;
pub mod message;
pub mod stats;
pub mod trunking;
pub mod voice;
