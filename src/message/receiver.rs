//! High-level receiver for receiving P25 voice, data, and trunking messages.

use error::P25Error;
use message::data_unit::{DataUnitReceiver, ReceiverEvent};
use message::nid::NetworkID;
use message::status::StreamSymbol;
use trunking::tsbk::{TSBKFields, TSBKReceiver};
use voice::control::LinkControlFields;
use voice::crypto::CryptoControlFields;
use voice::frame::VoiceFrame;
use voice::header::{VoiceHeaderReceiver, VoiceHeaderFields};
use voice::term::VoiceLCTerminatorReceiver;

use voice::frame_group::{
    FrameGroupEvent,
    VoiceCCFrameGroupReceiver,
    VoiceLCFrameGroupReceiver,
};

/// Set of callbacks to handle events and messages that occur when receiving the air
/// interface.
pub trait MessageHandler {
    /// A runtime error occured.
    fn handle_error(&mut self, recv: &mut DataUnitReceiver, err: P25Error);
    /// An NID was decoded.
    fn handle_nid(&mut self, recv: &mut DataUnitReceiver, nid: NetworkID);
    /// A voice header was received.
    fn handle_header(&mut self, recv: &mut DataUnitReceiver, header: VoiceHeaderFields);
    /// A voice frame was received.
    fn handle_frame(&mut self, recv: &mut DataUnitReceiver, frame: VoiceFrame);
    /// A link control word was decoded.
    fn handle_lc(&mut self, recv: &mut DataUnitReceiver, lc: LinkControlFields);
    /// A crypto control word was decoded.
    fn handle_cc(&mut self, recv: &mut DataUnitReceiver, cc: CryptoControlFields);
    /// A voice low-speed data fragment was decoded.
    fn handle_data_frag(&mut self, recv: &mut DataUnitReceiver, data: u32);
    /// A trunking signalling packet was received.
    fn handle_tsbk(&mut self, recv: &mut DataUnitReceiver, tsbk: TSBKFields);
    /// A voice terminator packet was received, optionally with link control word.
    fn handle_term(&mut self, recv: &mut DataUnitReceiver);
}

/// Internal state of the state machine.
enum State {
    /// Waiting for an event from the lower-level state machine.
    Idle,
    /// Decoding a voice header packet.
    DecodeHeader(VoiceHeaderReceiver),
    /// Decoding a link control frame group packet.
    DecodeLCFrameGroup(VoiceLCFrameGroupReceiver),
    /// Decoding a crypto control frame group packet.
    DecodeCCFrameGroup(VoiceCCFrameGroupReceiver),
    /// Decoding a link control voice terminator.
    DecodeLCTerminator(VoiceLCTerminatorReceiver),
    /// Decoding a trunking signalling packet.
    DecodeTSBK(TSBKReceiver),
}

/// State machine for high-level message reception.
pub struct MessageReceiver {
    /// Lower-level stream receiver.
    recv: DataUnitReceiver,
    /// Current state.
    state: State,
}

impl MessageReceiver {
    /// Create a new `MessageReceiver` in the initial state.
    pub fn new() -> MessageReceiver {
        MessageReceiver {
            recv: DataUnitReceiver::new(),
            state: State::Idle,
        }
    }

    pub fn resync(&mut self) { self.recv.resync(); }

    /// Feed in a baseband sample, possibly producing a new event or message to be handled
    /// by the given handler.
    pub fn feed<H: MessageHandler>(&mut self, s: f32, handler: &mut H) {
        use self::State::*;
        use message::nid::DataUnit::*;

        let event = match self.recv.feed(s) {
            Some(Ok(event)) => event,
            Some(Err(err)) => {
                handler.handle_error(&mut self.recv, err);
                self.recv.resync();

                return;
            },
            None => return,
        };

        let dibit = match event {
            ReceiverEvent::NetworkID(nid) => {
                self.state = match nid.data_unit {
                    VoiceHeader =>
                        DecodeHeader(VoiceHeaderReceiver::new()),
                    VoiceSimpleTerminator => {
                        handler.handle_term(&mut self.recv);
                        self.recv.flush_pads();
                        Idle
                    },
                    VoiceLCTerminator =>
                        DecodeLCTerminator(VoiceLCTerminatorReceiver::new()),
                    VoiceLCFrameGroup =>
                        DecodeLCFrameGroup(VoiceLCFrameGroupReceiver::new()),
                    VoiceCCFrameGroup =>
                        DecodeCCFrameGroup(VoiceCCFrameGroupReceiver::new()),
                    TrunkingSignaling =>
                        DecodeTSBK(TSBKReceiver::new()),
                    DataPacket => {
                        self.recv.resync();
                        Idle
                    },
                };

                handler.handle_nid(&mut self.recv, nid);

                return;
            },
            ReceiverEvent::Symbol(StreamSymbol::Status(_)) => return,
            ReceiverEvent::Symbol(StreamSymbol::Data(dibit)) => dibit,
        };

        match self.state {
            DecodeHeader(ref mut head) => match head.feed(dibit) {
                Some(Ok(h)) => {
                    handler.handle_header(&mut self.recv, h);
                    self.recv.flush_pads();
                },
                Some(Err(err)) => {
                    handler.handle_error(&mut self.recv, err);
                    self.recv.resync();
                },
                None => {},
            },
            DecodeLCFrameGroup(ref mut fg) => match fg.feed(dibit) {
                Some(Ok(event)) => match event {
                    FrameGroupEvent::VoiceFrame(vf) => {
                        handler.handle_frame(&mut self.recv, vf);

                        if fg.done() {
                            self.recv.flush_pads();
                        }
                    },
                    FrameGroupEvent::Extra(lc) => handler.handle_lc(&mut self.recv, lc),
                    FrameGroupEvent::DataFragment(data) =>
                        handler.handle_data_frag(&mut self.recv, data),
                },
                Some(Err(err)) => {
                    handler.handle_error(&mut self.recv, err);
                    self.recv.resync();
                },
                None => {},
            },
            DecodeCCFrameGroup(ref mut fg) => match fg.feed(dibit) {
                Some(Ok(event)) => match event {
                    FrameGroupEvent::VoiceFrame(vf) => {
                        handler.handle_frame(&mut self.recv, vf);

                        if fg.done() {
                            self.recv.flush_pads();
                        }
                    },
                    FrameGroupEvent::Extra(cc) => handler.handle_cc(&mut self.recv, cc),
                    FrameGroupEvent::DataFragment(data) =>
                        handler.handle_data_frag(&mut self.recv, data),
                },
                Some(Err(err)) => {
                    handler.handle_error(&mut self.recv, err);
                    self.recv.resync();
                },
                None => {},
            },
            DecodeLCTerminator(ref mut term) => match term.feed(dibit) {
                Some(Ok(lc)) => {
                    handler.handle_lc(&mut self.recv, lc);
                    handler.handle_term(&mut self.recv);
                    self.recv.flush_pads();
                },
                Some(Err(err)) => {
                    handler.handle_error(&mut self.recv, err);
                    self.recv.resync();
                },
                None => {},
            },
            DecodeTSBK(ref mut dec) => match dec.feed(dibit) {
                Some(Ok(tsbk)) => {
                    handler.handle_tsbk(&mut self.recv, tsbk);

                    if tsbk.is_tail() {
                        self.recv.flush_pads();
                    }
                },
                Some(Err(err)) => {
                    handler.handle_error(&mut self.recv, err);
                    self.recv.resync();
                },
                None => {},
            },
            Idle => {},
        }
    }
}
