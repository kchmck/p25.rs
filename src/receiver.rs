use baseband::{Decoder, Decider, Correlator};
use error::{P25Error, Result};
use nid;
use status::{StreamSymbol, StatusDeinterleaver};
use sync::{SyncCorrelator, SyncDetector};

use self::State::*;
use self::StateChange::*;

#[derive(Debug)]
pub enum ReceiverEvent {
    Symbol(StreamSymbol),
    NetworkID(nid::NetworkID),
}

#[derive(Copy, Clone)]
struct Receiver {
    recv: Decoder,
    status: StatusDeinterleaver,
}

impl Receiver {
    pub fn new(recv: Decoder) -> Receiver {
        Receiver {
            recv: recv,
            status: StatusDeinterleaver::new(),
        }
    }

    pub fn feed(&mut self, s: f32) -> Option<StreamSymbol> {
        let dibit = match self.recv.feed(s) {
            Some(dibit) => dibit,
            None => return None,
        };

        Some(self.status.feed(dibit))
    }
}

enum State {
    Prime(u32),
    Sync(SyncDetector),
    DecodeNID(Receiver, nid::NIDReceiver),
    DecodePacket(Receiver),
    FlushPads(Receiver),
}

enum StateChange {
    Change(State),
    Event(ReceiverEvent),
    EventChange(ReceiverEvent, State),
    Error(P25Error),
    NoChange,
}

impl State {
    pub fn prime() -> State { Prime(0) }
    pub fn sync() -> State { Sync(SyncDetector::new()) }
    pub fn decode_nid(decoder: Decoder) -> State {
        DecodeNID(Receiver::new(decoder), nid::NIDReceiver::new())
    }
    pub fn decode_packet(recv: Receiver) -> State { DecodePacket(recv) }
    pub fn flush_pads(recv: Receiver) -> State { FlushPads(recv) }
}

pub struct DataUnitReceiver {
    state: State,
    corr: SyncCorrelator,
}

const PRIME_SAMPLES: u32 = 1000;

impl DataUnitReceiver {
    pub fn new() -> DataUnitReceiver {
        DataUnitReceiver {
            state: State::prime(),
            corr: SyncCorrelator::new(),
        }
    }

    pub fn flush_pads(&mut self) {
        if let DecodePacket(recv) = self.state {
            self.state = State::flush_pads(recv);
        } else {
            panic!("not decoding a packet");
        }
    }

    pub fn resync(&mut self) { self.state = State::sync(); }

    fn handle(&mut self, s: f32) -> StateChange {
        let energy = self.corr.feed(s);

        println!("energy {}", energy);

        match self.state {
            Prime(t) => if t == PRIME_SAMPLES - 1 {
                Change(State::sync())
            } else {
                Change(Prime(t + 1))
            },
            Sync(ref mut sync) => match sync.feed(energy, self.corr.thresh()) {
                Some(thresh) => {
                    println!("FOUND SYNC");
                    self.corr.print();
                    Change(State::decode_nid(
                    Decoder::new(Correlator::primed(s), Decider::new(thresh))))
                },
                None => NoChange,
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
                Some(StreamSymbol::Status(_)) => Change(State::sync()),
                _ => NoChange,
            },
        }
    }

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
