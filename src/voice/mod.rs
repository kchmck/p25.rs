//! Decoding of voice-related data units.

mod descramble;
mod frame_group;
mod rand;

pub mod control;
pub mod crypto;
pub mod frame;
pub mod header;
pub mod term;

pub use self::frame_group::{VoiceLCFrameGroupReceiver, VoiceCCFrameGroupReceiver,
                            FrameGroupEvent};
