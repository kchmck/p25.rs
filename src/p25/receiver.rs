use bits;
use sync;
use baseband::Decoder;

use self::ReceiveState::*;

enum ReceiveState {
    Syncing(sync::SyncDetector),
    Receiving(Decoder),
    Dibit(bits::Dibit),
}

pub struct Receiver {
    state: ReceiveState,
}

impl Receiver {
    pub fn new(period: usize) -> Receiver {
        Receiver {
            state: Syncing(sync::SyncDetector::new(period)),
        }
    }

    fn handle(&mut self, s: f64, t: usize) -> Option<ReceiveState> {
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
