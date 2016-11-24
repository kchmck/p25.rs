//! General low-level receiver for all data units, covering frame synchronization up to
//! symbol decoding.

use baseband::decode::{Decoder, Decider};
use baseband::sync::{SyncCorrelator, SyncDetector};
use error::{P25Error, Result};
use message::nid;
use message::status::{StreamSymbol, StatusDeinterleaver};

use self::State::*;
use self::StateChange::*;

/// Number of samples used to initially prime the correlator's moving average before starting to
/// look for sync.
const PRIME_SAMPLES: u32 = 6000;

/// Low-level receiver for decoding samples into symbols and deinterleaving status
/// symbols.
#[derive(Copy, Clone)]
struct SymbolReceiver {
    /// Symbol decoder.
    decoder: Decoder,
    /// Data/Status symbol deinterleaver.
    status: StatusDeinterleaver,
}

impl SymbolReceiver {
    /// Create a new `SymbolReceiver` using the given symbol decoder.
    pub fn new(decoder: Decoder) -> SymbolReceiver {
        SymbolReceiver {
            decoder: decoder,
            status: StatusDeinterleaver::new(),
        }
    }

    /// Feed in a baseband symbol, possibly producing a data or status symbol.
    pub fn feed(&mut self, s: f32) -> Option<StreamSymbol> {
        match self.decoder.feed(s) {
            Some(dibit) => Some(self.status.feed(dibit)),
            None => None,
        }
    }
}


/// An event seen by the low-level receiver.
#[derive(Debug)]
pub enum ReceiverEvent {
    /// Data or status symbol.
    Symbol(StreamSymbol),
    /// Decoded NID information.
    NetworkID(nid::NetworkID),
}

/// Internal state of the state machine.
enum State {
    /// Prime the signal power tracker.
    Prime(u32),
    /// Lock onto frame synchronization.
    Sync(SyncDetector),
    /// Decode NID.
    DecodeNID(SymbolReceiver, nid::NIDReceiver),
    /// Decode data and status symbols.
    DecodePacket(SymbolReceiver),
    /// Flush pads at end of packet.
    FlushPads(SymbolReceiver),
}

/// Action the state machine should take.
enum StateChange {
    /// Change to the given state.
    Change(State),
    /// Propagate the given event.
    Event(ReceiverEvent),
    /// Change to the given state and propagate the given event.
    EventChange(ReceiverEvent, State),
    /// Propagate the given error.
    Error(P25Error),
    /// No action necessary.
    NoChange,
}

impl State {
    /// Initial prime state.
    pub fn prime() -> State { Prime(1) }

    /// Initial synchronization state.
    pub fn sync() -> State { Sync(SyncDetector::new()) }

    /// Initial NID decode state.
    pub fn decode_nid(decoder: Decoder) -> State {
        DecodeNID(SymbolReceiver::new(decoder), nid::NIDReceiver::new())
    }

    /// Initial symbol decode state.
    pub fn decode_packet(recv: SymbolReceiver) -> State { DecodePacket(recv) }

    /// Initial flush padding state.
    pub fn flush_pads(recv: SymbolReceiver) -> State { FlushPads(recv) }
}

/// State machine for low-level data unit reception.
///
/// The state machine consumes baseband samples and performs the following steps common to
/// all data units:
///
/// 1. Track average power of input signal
/// 2. Lock onto frame synchronization
/// 3. Deinterleave status symbols
/// 4. Decode NID information
/// 5. Decode dibit symbols until stopped
pub struct DataUnitReceiver {
    /// Current state.
    state: State,
    /// Tracks input signal power and frame synchronization statistics.
    corr: SyncCorrelator,
}

impl DataUnitReceiver {
    /// Create a new `DataUnitReceiver` in the initial reception state.
    pub fn new() -> DataUnitReceiver {
        DataUnitReceiver {
            state: State::prime(),
            corr: SyncCorrelator::new(),
        }
    }

    /// Flush any remaining padding symbols at the end of the current packet, and reenter
    /// the frame synchronization state afterwards.
    pub fn flush_pads(&mut self) {
        match self.state {
            DecodePacket(recv) => self.state = State::flush_pads(recv),
            Sync(_) => {},
            _ => panic!("not decoding a packet"),
        }
    }

    /// Force the receiver into frame synchronization.
    pub fn resync(&mut self) { self.state = State::sync(); }

    /// Determine the next action to take based on the given sample.
    fn handle(&mut self, s: f32) -> StateChange {
        // Continuously track the input signal power.
        let (power, thresh) = self.corr.feed(s);

        match self.state {
            Prime(t) => if t == PRIME_SAMPLES {
                Change(State::sync())
            } else {
                Change(Prime(t + 1))
            },
            Sync(ref mut sync) => if sync.feed(power, thresh) {
                let (p, m, n) = self.corr.thresholds();
                Change(State::decode_nid(Decoder::new(Decider::new(p, m, n))))
            } else {
                NoChange
            },
            DecodeNID(ref mut recv, ref mut nid) => {
                let dibit = match recv.feed(s) {
                    Some(StreamSymbol::Data(d)) => d,
                    Some(s) => return Event(ReceiverEvent::Symbol(s)),
                    None => return NoChange,
                };

                match nid.feed(dibit) {
                    Some(Ok(nid)) => EventChange(ReceiverEvent::NetworkID(nid),
                                                 State::decode_packet(*recv)),
                    Some(Err(e)) => Error(e),
                    None => NoChange,
                }
            },
            DecodePacket(ref mut recv) => match recv.feed(s) {
                Some(x) => Event(ReceiverEvent::Symbol(x)),
                None => NoChange,
            },
            FlushPads(ref mut recv) => match recv.feed(s) {
                /// According to the spec, the stream is padded until the next status
                /// symbol boundary.
                Some(StreamSymbol::Status(_)) => Change(State::sync()),
                _ => NoChange,
            },
        }
    }

    /// Feed in a baseband symbol, possibly producing a receiver event. Return
    /// `Some(Ok(event))` for any normal event, `Some(Err(err))` for any error, and `None`
    /// if no event occurred.
    pub fn feed(&mut self, s: f32) -> Option<Result<ReceiverEvent>> {
        match self.handle(s) {
            Change(state) => {
                self.state = state;
                None
            },
            Event(event) => Some(Ok(event)),
            EventChange(event, state) => {
                self.state = state;
                Some(Ok(event))
            },
            Error(err) => Some(Err(err)),
            NoChange => None,
        }
    }
}
