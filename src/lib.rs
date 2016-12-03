//! Implements the Project (P25) air interface radio protocol, including baseband frame
//! synchronization, symbol decoding, error correction coding, and packet reconstuction.

#![feature(const_fn)]
#![feature(inclusive_range_syntax)]

extern crate collect_slice;
extern crate num;
extern crate static_ewma;

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
pub mod trunking;
pub mod voice;
