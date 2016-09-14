#![feature(const_fn)]

extern crate collect_slice;
extern crate static_ewma;
extern crate num;

#[macro_use]
extern crate static_fir;

mod buffer;
mod util;

pub mod baseband;
pub mod bits;
pub mod c4fm;
pub mod coding;
pub mod consts;
pub mod data;
pub mod error;
pub mod message;
pub mod nid;
pub mod receiver;
pub mod status;
pub mod sync;
pub mod trunking;
pub mod voice;
