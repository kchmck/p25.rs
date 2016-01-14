use bits;
use self::StatusCode::*;
use self::StreamSymbol::*;

/// Number of dibits output before the 2-bit status symbol.
const DIBITS_PER_UPDATE: u32 = 70 / 2;

pub trait StatusSource {
    fn status(&mut self) -> StatusCode;
}

pub struct StatusInterleaver<T, S> where
    T: Iterator<Item = bits::Dibit>,
    S: StatusSource
{
    /// Source of dibits to interleave status symbols into.
    src: T,
    /// Source of status updates.
    status: S,
    /// Current dibit index in output stream.
    pos: u32,
}

impl<T, S> StatusInterleaver<T, S> where
    T: Iterator<Item = bits::Dibit>,
    S: StatusSource
{
    pub fn new(src: T, status: S) -> StatusInterleaver<T, S> {
        StatusInterleaver {
            status: status,
            src: src,
            pos: 0,
        }
    }
}

impl<T, S> Iterator for StatusInterleaver<T, S> where
    T: Iterator<Item = bits::Dibit>,
    S: StatusSource
{
    type Item = bits::Dibit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == DIBITS_PER_UPDATE {
            self.pos = 0;
            return Some(self.status.status().to_dibit());
        }

        self.pos += 1;

        match self.src.next() {
            Some(d) => Some(d),
            // If just after an update and no more source dibits, end the iteration.
            None if self.pos == 1 => None,
            // Pad until the next update.
            None => Some(bits::Dibit::new(0b00)),
        }
    }
}

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
    pub fn from_dibit(d: bits::Dibit) -> StatusCode {
        match d.bits() {
            0b01 => InboundBusy,
            0b00 => SubscriberDirect,
            0b10 => SubscriberRepeater,
            0b11 => InboundIdle,
            _ => unreachable!(),
        }
    }

    pub fn to_dibit(&self) -> bits::Dibit {
        bits::Dibit::new(match *self {
            InboundBusy => 0b01,
            SubscriberDirect => 0b00,
            SubscriberRepeater => 0b10,
            InboundIdle => 0b11,
        })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StreamSymbol {
    Status(StatusCode),
    Data(bits::Dibit),
}

#[derive(Copy, Clone)]
pub struct StatusDeinterleaver {
    pos: u32,
}

impl StatusDeinterleaver {
    pub fn new() -> StatusDeinterleaver {
        StatusDeinterleaver {
            pos: 24,
        }
    }

    fn reset(&mut self) { self.pos = 0; }

    pub fn feed(&mut self, d: bits::Dibit) -> StreamSymbol {
        if self.pos == DIBITS_PER_UPDATE {
            self.reset();
            Status(StatusCode::from_dibit(d))
        } else {
            self.pos += 1;
            Data(d)
        }
    }
}

#[cfg(test)]
mod test {
    use bits;
    use super::*;

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
