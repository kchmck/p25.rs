//! Provides a convenience interface for coding symbols into a buffer.

use bits;
use coding::trellis;
use data::consts;

/// Half-rate (dibit) convolutional coder.
pub type DibitCoder = DataCoder<trellis::DibitStates>;

/// 3/4-rate (tribit) convolutional coder.
pub type TribitCoder = DataCoder<trellis::TribitStates>;

pub struct DataCoder<S: trellis::States> {
    /// Convolutional state machine.
    fsm: trellis::TrellisFSM<S>,
    /// Current coded buffer.
    buf: [bits::Dibit; consts::CODING_DIBITS],
    /// Current index into `buf`.
    pos: usize,
}

impl<S: trellis::States> DataCoder<S> {
    /// Construct a new `DataCoder` wrapping the given state machine.
    fn for_fsm(fsm: trellis::TrellisFSM<S>) -> DataCoder<S> {
        DataCoder {
            fsm: fsm,
            buf: [bits::Dibit::default(); consts::CODING_DIBITS],
            pos: 0,
        }
    }

    /// Flush the state machine and return the coded buffer of dibits.
    pub fn finish(mut self) -> [bits::Dibit; consts::CODING_DIBITS] {
        let pair = self.fsm.finish();
        self.append(pair);

        assert!(self.pos == self.buf.len());

        self.buf
    }

    /// Code the given symbol and add the result to the buffer.
    fn feed_symbol(&mut self, symbol: S::Symbol) {
        let pair = self.fsm.feed(symbol);
        self.append(pair);
    }

    /// Append the given dibit pair to the buffer.
    fn append(&mut self, (a, b): (bits::Dibit, bits::Dibit)) {
        self.buf[self.pos] = a;
        self.pos += 1;
        self.buf[self.pos] = b;
        self.pos += 1;
    }
}

impl DibitCoder {
    /// Construct a new `DibitCoder` for coding a dibit stream.
    pub fn new() -> DibitCoder {
        Self::for_fsm(trellis::DibitFSM::new())
    }

    /// Code the given bytes as dibits.
    pub fn feed_bytes<T: Iterator<Item = u8>>(mut self, bytes: T) -> Self {
        for dibit in bits::Dibits::new(bytes) {
            self.feed_symbol(dibit);
        }

        self
    }
}

impl TribitCoder {
    /// Construct a new `TribitCoder` for coding a tribit stream.
    pub fn new() -> TribitCoder {
        Self::for_fsm(trellis::TribitFSM::new())
    }

    /// Code the given bytes as tribits.
    pub fn feed_bytes<T: Iterator<Item = u8>>(mut self, bytes: T) -> Self {
        for tribit in bits::Tribits::new(bytes) {
            self.feed_symbol(tribit);
        }

        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dibit_coder() {
        let buf = DibitCoder::new().feed_bytes((0..12).map(|i| {
            if i % 2 == 0 { 0x00 } else { 0xFF }
        })).finish();

        assert_eq!(buf[0].bits(), 0b00);
        assert_eq!(buf[1].bits(), 0b10);
        assert_eq!(buf[2].bits(), 0b00);
        assert_eq!(buf[3].bits(), 0b10);
        assert_eq!(buf[4].bits(), 0b00);
        assert_eq!(buf[5].bits(), 0b10);
        assert_eq!(buf[6].bits(), 0b00);
        assert_eq!(buf[7].bits(), 0b10);

        assert_eq!(buf[8].bits(), 0b11);
        assert_eq!(buf[9].bits(), 0b11);
        assert_eq!(buf[10].bits(), 0b10);
        assert_eq!(buf[11].bits(), 0b00);
        assert_eq!(buf[12].bits(), 0b10);
        assert_eq!(buf[13].bits(), 0b00);
        assert_eq!(buf[14].bits(), 0b10);
        assert_eq!(buf[15].bits(), 0b00);
    }

    #[test]
    fn test_tribit_coder() {
        let buf = TribitCoder::new().feed_bytes((0..18).map(|i| {
            if i % 2 == 0 { 0x00 } else { 0xFF }
        })).finish();

        assert_eq!(buf[0].bits(), 0b00);
        assert_eq!(buf[1].bits(), 0b10);

        assert_eq!(buf[2].bits(), 0b00);
        assert_eq!(buf[3].bits(), 0b10);

        assert_eq!(buf[4].bits(), 0b11);
        assert_eq!(buf[5].bits(), 0b01);

        assert_eq!(buf[6].bits(), 0b11);
        assert_eq!(buf[7].bits(), 0b01);

        assert_eq!(buf[8].bits(), 0b10);
        assert_eq!(buf[9].bits(), 0b00);

        assert_eq!(buf[10].bits(), 0b11);
        assert_eq!(buf[11].bits(), 0b10);

        assert_eq!(buf[12].bits(), 0b11);
        assert_eq!(buf[13].bits(), 0b11);
    }
}
