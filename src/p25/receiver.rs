use std;

use bits;
use sync;
use system::{SystemParams, P25Params};
use baseband::Decoder;

use self::ReceiveState::*;

enum ReceiveState<S: SystemParams> {
    Syncing(sync::SyncDetector<S>),
    Receiving(Decoder<S>),
    Dibit(bits::Dibit),
}

pub struct Receiver<S: SystemParams = P25Params> {
    system: std::marker::PhantomData<S>,
    state: ReceiveState<S>,
}

impl<S: SystemParams = P25Params> Receiver<S> {
    pub fn new() -> Receiver<S> {
        Receiver {
            system: std::marker::PhantomData,
            state: Syncing(sync::SyncDetector::new()),
        }
    }

    fn handle(&mut self, s: f64, t: usize) -> Option<ReceiveState<S>> {
        match self.state {
            Syncing(ref mut sync) => match sync.feed(s, t) {
                Some(Err(_)) => {
                    sync.reset();
                    None
                },
                Some(Ok(decoder)) => Some(Receiving(decoder)),
                None => None,
            },
            Receiving(ref mut decoder) => match decoder.feed(s) {
                Some(d) => Some(Dibit(d)),
                None => None,
            },
            Dibit(_) => panic!(),
        }
    }

    pub fn feed(&mut self, s: f64, t: usize) -> Option<bits::Dibit> {
        match self.handle(s, t) {
            Some(Dibit(d)) => Some(d),
            Some(next) => {
                self.state = next;
                None
            },
            None => None,
        }
    }
}
