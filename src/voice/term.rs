//! Decode voice Link Control (LC) terminator packets.

use collect_slice::CollectSlice;

use bits::{Dibit, Hexbit, HexbitBytes};
use buffer::{Buffer, VoiceLCTermWordStorage, VoiceExtraStorage};
use coding::{reed_solomon, golay};
use consts::LINK_CONTROL_BYTES;
use error::Result;
use stats::{Stats, HasStats};
use voice::control::LinkControlFields;

use error::P25Error::*;

/// State machine for receiving a link control voice terminator.
pub struct VoiceLCTerminatorReceiver {
    /// Current buffered dibits for the current hexbit.
    outer: Buffer<VoiceLCTermWordStorage>,
    /// Current buffered hexbits.
    inner: Buffer<VoiceExtraStorage>,
    stats: Stats,
}

impl VoiceLCTerminatorReceiver {
    /// Create a new `VoiceLCTerminatorReceiver` in the initial state.
    pub fn new() -> VoiceLCTerminatorReceiver {
        VoiceLCTerminatorReceiver {
            outer: Buffer::new(VoiceLCTermWordStorage::new()),
            inner: Buffer::new(VoiceExtraStorage::new()),
            stats: Stats::default(),
        }
    }

    /// Feed in a baseband symbol, possibly producing a link control packet. Return
    /// `Some(Ok(lc))` if an LC packet was successfully recovered from the terminator,
    /// `Some(Err(err))` if an error occurred, and `None` in the case of no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<LinkControlFields>> {
        let buf = match self.outer.feed(dibit) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match golay::extended::decode(*buf as u32) {
            Some((data, err)) => {
                self.stats.record_golay_ext(err);
                data
            },
            // Let the following RS code attempt to correct these errors.
            None => 0,
        };

        // Each 12-bit word is turned into 2 hexbits.
        assert!(self.inner.feed(Hexbit::new((data >> 6) as u8)).is_none());

        let hexbits = match self.inner.feed(Hexbit::new((data & 0x3F) as u8)) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match reed_solomon::short::decode(hexbits) {
            Some((data, err)) => {
                self.stats.record_rs_short(err);
                data
            },
            None => return Some(Err(RsShortUnrecoverable)),
        };

        let mut bytes = [0; LINK_CONTROL_BYTES];
        HexbitBytes::new(data.iter().cloned())
            .collect_slice_checked(&mut bytes[..]);

        Some(Ok(LinkControlFields::new(bytes)))
    }
}

impl HasStats for VoiceLCTerminatorReceiver {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}
