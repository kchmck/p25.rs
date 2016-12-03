//! Status symbol interleaving and deinterleaving.

use bits;
use consts::SYNC_SYMBOLS;

use self::StatusCode::*;
use self::StreamSymbol::*;

/// Number of dibits output per status period, including the status symbol.
const DIBITS_PER_UPDATE: u32 = 70 / 2 + 1;

/// Interleaves status symbols into a stream of dibits.
pub struct StatusInterleaver<T: Iterator<Item = bits::Dibit>> {
    /// Source of dibits to interleave status symbols into.
    src: T,
    /// Current status to be output.
    status: StatusCode,
    /// Current dibit index in output stream.
    pos: u32,
}

impl<T: Iterator<Item = bits::Dibit>> StatusInterleaver<T> {
    /// Create a new `StatusInterleaver` that will interleave status symbols into the
    /// given source of data symbols, using the given initial status code.
    pub fn new(src: T, status: StatusCode) -> StatusInterleaver<T> {
        StatusInterleaver {
            src: src,
            status: status,
            pos: 0,
        }
    }

    /// Update current output status to the given status code.
    pub fn update_status(&mut self, status: StatusCode) { self.status = status; }
}

impl<T: Iterator<Item = bits::Dibit>> Iterator for StatusInterleaver<T> {
    type Item = bits::Dibit;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos += 1;
        self.pos %= DIBITS_PER_UPDATE;

        if self.pos == 0 {
            return Some(self.status.to_dibit());
        }

        match self.src.next() {
            Some(d) => Some(d),
            // If just after an update and no more source dibits, end the iteration.
            None if self.pos == 1 => None,
            // Pad until the next update.
            None => Some(bits::Dibit::new(0b00)),
        }
    }
}

/// A P25 status symbol.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StatusCode {
    /// Used by a repeater when the inbound channel is idle.
    InboundIdle,
    /// Used by a repeater when the inbound channel is busy.
    InboundBusy,
    /// Used when a subscriber is transmitting to a repeater.
    SubscriberRepeater,
    /// Used when a subscriber is transmitting directly to another subscriber.
    SubscriberDirect,
}

impl StatusCode {
    /// Parse a status code from the given dibit.
    pub fn from_dibit(d: bits::Dibit) -> StatusCode {
        match d.bits() {
            0b01 => InboundBusy,
            0b00 => SubscriberDirect,
            0b10 => SubscriberRepeater,
            0b11 => InboundIdle,
            _ => unreachable!(),
        }
    }

    /// Convert the current status code into a dibit.
    pub fn to_dibit(self) -> bits::Dibit {
        bits::Dibit::new(match self {
            InboundBusy => 0b01,
            SubscriberDirect => 0b00,
            SubscriberRepeater => 0b10,
            InboundIdle => 0b11,
        })
    }
}

/// A symbol in a transmitted P25 stream.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StreamSymbol {
    /// Current symbol is a status code.
    Status(StatusCode),
    /// Current symbol is a data dibit.
    Data(bits::Dibit),
}

/// Deinterleave a P25 transmitted stream into status codes and data symbols.
#[derive(Copy, Clone)]
pub struct StatusDeinterleaver {
    /// Current dibit position in current status period.
    pos: u32,
}

impl StatusDeinterleaver {
    /// Create a new `StatusDeinterleaver` for deinterlacing immediately after the frame sync
    /// sequence.
    pub fn new() -> StatusDeinterleaver {
        StatusDeinterleaver {
            // Since stream deinterleaving is started after the frame sync, and the frame sync
            // symbols count towards the first status symbol period, start the counter with those
            // symbols taken into account.
            pos: SYNC_SYMBOLS as u32,
        }
    }

    /// Parse the given symbol as a status or data symbol.
    pub fn feed(&mut self, d: bits::Dibit) -> StreamSymbol {
        self.pos += 1;
        self.pos %= DIBITS_PER_UPDATE;

        if self.pos == 0 {
            Status(StatusCode::from_dibit(d))
        } else {
            Data(d)
        }
    }
}

#[cfg(test)]
mod test {
    use bits;
    use super::*;
    use std;

    #[test]
    fn test_interleave() {
        let src = std::iter::repeat(bits::Dibit::new(0b10));
        let mut i = StatusInterleaver::new(src, StatusCode::InboundBusy);

        for _ in 0..35 {
            assert_eq!(i.next(), Some(bits::Dibit::new(0b10)));
        }

        assert_eq!(i.next(), Some(bits::Dibit::new(0b01)));

        for _ in 0..35 {
            assert_eq!(i.next(), Some(bits::Dibit::new(0b10)));
        }

        assert_eq!(i.next(), Some(bits::Dibit::new(0b01)));
    }

    #[test]
    fn test_deinterleave() {
        let mut d = StatusDeinterleaver::new();

        for _ in 0..11 {
            assert_eq!(d.feed(bits::Dibit::new(0)),
                StreamSymbol::Data(bits::Dibit::new(0)));
        }

        assert_eq!(d.feed(bits::Dibit::new(0)), StreamSymbol::Status(
                StatusCode::SubscriberDirect));

        for _ in 0..35 {
            assert_eq!(d.feed(bits::Dibit::new(0)),
                StreamSymbol::Data(bits::Dibit::new(0)));
        }

        assert_eq!(d.feed(bits::Dibit::new(0)), StreamSymbol::Status(
                StatusCode::SubscriberDirect));

        for _ in 0..35 {
            assert_eq!(d.feed(bits::Dibit::new(0)),
                StreamSymbol::Data(bits::Dibit::new(0)));
        }

        assert_eq!(d.feed(bits::Dibit::new(0)), StreamSymbol::Status(
                StatusCode::SubscriberDirect));
    }
}
