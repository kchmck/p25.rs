use collect_slice::CollectSlice;

use bits::{Dibit, Hexbit, HexbitBytes};
use buffer::{Buffer, VoiceLCTermWordStorage, VoiceHeaderWordStorage, VoiceHeaderStorage,
             VoiceExtraStorage};
use coding::{reed_solomon, golay};
use consts::{HEADER_BYTES, LINK_CONTROL_BYTES};
use error::Result;

use voice::header::VoiceHeaderFields;
use voice::control::LinkControlFields;

use error::P25Error::*;

pub struct VoiceHeaderReceiver {
    dibits: Buffer<VoiceHeaderWordStorage>,
    hexbits: Buffer<VoiceHeaderStorage>,
}

impl VoiceHeaderReceiver {
    pub fn new() -> VoiceHeaderReceiver {
        VoiceHeaderReceiver {
            dibits: Buffer::new(VoiceHeaderWordStorage::new()),
            hexbits: Buffer::new(VoiceHeaderStorage::new()),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<VoiceHeaderFields>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u32,
            None => return None,
        };

        let data = match golay::shortened::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(GolayUnrecoverable)),
        };

        let hexbits = match self.hexbits.feed(Hexbit::new(data)) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match reed_solomon::long::decode(hexbits) {
            Some((data, err)) => data,
            None => return Some(Err(ReedSolomonUnrecoverable)),
        };

        let mut bytes = [0; HEADER_BYTES];
        HexbitBytes::new(data.iter().cloned())
            .collect_slice_checked(&mut bytes[..]);

        Some(Ok(VoiceHeaderFields::new(bytes)))
    }
}

pub struct VoiceLCTerminatorReceiver {
    outer: Buffer<VoiceLCTermWordStorage>,
    inner: Buffer<VoiceExtraStorage>,
}

impl VoiceLCTerminatorReceiver {
    pub fn new() -> VoiceLCTerminatorReceiver {
        VoiceLCTerminatorReceiver {
            outer: Buffer::new(VoiceLCTermWordStorage::new()),
            inner: Buffer::new(VoiceExtraStorage::new()),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<LinkControlFields>> {
        let buf = match self.outer.feed(dibit) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match golay::extended::decode(*buf as u32) {
            Some((data, err)) => data,
            None => return Some(Err(GolayUnrecoverable)),
        };

        assert!(self.inner.feed(Hexbit::new((data >> 6) as u8)).is_none());

        let hexbits = match self.inner.feed(Hexbit::new((data & 0x3F) as u8)) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match reed_solomon::short::decode(hexbits) {
            Some((data, err)) => data,
            None => return Some(Err(ReedSolomonUnrecoverable)),
        };

        let mut bytes = [0; LINK_CONTROL_BYTES];
        HexbitBytes::new(data.iter().cloned())
            .collect_slice_checked(&mut bytes[..]);

        Some(Ok(LinkControlFields::new(bytes)))
    }
}
