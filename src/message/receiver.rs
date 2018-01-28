//! High-level receiver for receiving P25 voice, data, and trunking messages.

use error::P25Error;
use message::data_unit::{DataUnitReceiver, ReceiverEvent};
use message::nid::NetworkId;
use message::status::StreamSymbol;
use trunking::tsbk::{TsbkFields, TsbkReceiver};
use voice::control::LinkControlFields;
use voice::crypto::CryptoControlFields;
use voice::frame::VoiceFrame;
use voice::header::{VoiceHeaderReceiver, VoiceHeaderFields};
use voice::term::VoiceLCTerminatorReceiver;
use stats::{Stats, HasStats};

use voice::frame_group::{
    FrameGroupEvent,
    VoiceCCFrameGroupReceiver,
    VoiceLCFrameGroupReceiver,
};

/// Events that can occur when receiving P25 messages.
pub enum MessageEvent {
    /// A runtime error occured.
    Error(P25Error),
    /// An NID at the start of a packet was decoded.
    PacketNID(NetworkId),
    /// A voice header was received.
    VoiceHeader(VoiceHeaderFields),
    /// A voice frame was received.
    VoiceFrame(VoiceFrame),
    /// A link control word was decoded.
    LinkControl(LinkControlFields),
    /// A crypto control word was decoded.
    CryptoControl(CryptoControlFields),
    /// A voice low-speed data fragment was decoded.
    LowSpeedDataFragment(u32),
    /// A trunking signalling packet was received.
    TrunkingControl(TsbkFields),
    /// A voice terminator link control was received.
    VoiceTerm(LinkControlFields),
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
    DecodeTSBK(TsbkReceiver),
}

/// Action the state machine should take.
enum StateChange {
    /// Propagate an event.
    Event(MessageEvent),
    /// Propagate an event and change state.
    EventChange(MessageEvent, State),
    /// Do nothing.
    NoChange,
}

/// State machine for high-level message reception.
pub struct MessageReceiver {
    /// Lower-level stream receiver.
    recv: DataUnitReceiver,
    /// Current state.
    state: State,
    stats: Stats,
}

impl MessageReceiver {
    /// Create a new `MessageReceiver` in the initial state.
    pub fn new() -> MessageReceiver {
        MessageReceiver {
            recv: DataUnitReceiver::new(),
            state: State::Idle,
            stats: Stats::default(),
        }
    }

    /// Force the receiver into frame synchronization.
    pub fn resync(&mut self) { self.recv.resync(); }

    /// Feed in a baseband sample, possibly producing a new event or message to be handled
    /// by the given handler.
    pub fn feed(&mut self, s: f32) -> Option<MessageEvent> {
        match self.handle(s) {
            StateChange::Event(e) => Some(e),
            StateChange::EventChange(e, s) => {
                self.state = s;
                Some(e)
            },
            StateChange::NoChange => None,
        }
    }

    /// Process the given sample and determine how to update state.
    fn handle(&mut self, s: f32) -> StateChange {
        use self::State::*;
        use self::StateChange::*;
        use message::nid::DataUnit::*;

        let event = match self.recv.feed(s) {
            Some(Ok(event)) => event,
            Some(Err(err)) => {
                self.recv.resync();
                return Event(MessageEvent::Error(err));
            },
            None => return NoChange,
        };

        self.stats.merge(&mut self.recv);

        let dibit = match event {
            ReceiverEvent::NetworkId(nid) => {
                let next = match nid.data_unit {
                    VoiceHeader =>
                        DecodeHeader(VoiceHeaderReceiver::new()),
                    VoiceSimpleTerminator => {
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
                        DecodeTSBK(TsbkReceiver::new()),
                    DataPacket => {
                        self.recv.resync();
                        Idle
                    },
                };

                return EventChange(MessageEvent::PacketNID(nid), next);
            },
            ReceiverEvent::Symbol(StreamSymbol::Status(_)) => return NoChange,
            ReceiverEvent::Symbol(StreamSymbol::Data(dibit)) => dibit,
        };

        let next = match self.state {
            DecodeHeader(ref mut head) => match head.feed(dibit) {
                Some(Ok(h)) => {
                    self.recv.flush_pads();
                    EventChange(MessageEvent::VoiceHeader(h), Idle)
                },
                Some(Err(err)) => {
                    self.recv.resync();
                    EventChange(MessageEvent::Error(err), Idle)
                },
                None => NoChange,
            },
            DecodeLCFrameGroup(ref mut fg) => match fg.feed(dibit) {
                Some(Ok(event)) => {
                    if fg.done() {
                        self.recv.flush_pads();
                    }

                    match event {
                        FrameGroupEvent::VoiceFrame(vf) =>
                            Event(MessageEvent::VoiceFrame(vf)),
                        FrameGroupEvent::Extra(lc) =>
                            Event(MessageEvent::LinkControl(lc)),
                        FrameGroupEvent::DataFragment(frag) =>
                            Event(MessageEvent::LowSpeedDataFragment(frag)),
                    }
                },
                Some(Err(err)) => {
                    self.recv.resync();
                    EventChange(MessageEvent::Error(err), Idle)
                },
                None => NoChange,
            },
            DecodeCCFrameGroup(ref mut fg) => match fg.feed(dibit) {
                Some(Ok(event)) => match event {
                    FrameGroupEvent::VoiceFrame(vf) => {
                        if fg.done() {
                            self.recv.flush_pads();
                        }

                        Event(MessageEvent::VoiceFrame(vf))
                    },
                    FrameGroupEvent::Extra(cc) =>
                        Event(MessageEvent::CryptoControl(cc)),
                    FrameGroupEvent::DataFragment(frag) =>
                        Event(MessageEvent::LowSpeedDataFragment(frag))
                },
                Some(Err(err)) => {
                    self.recv.resync();
                    EventChange(MessageEvent::Error(err), Idle)
                },
                None => NoChange,
            },
            DecodeLCTerminator(ref mut term) => match term.feed(dibit) {
                Some(Ok(lc)) => {
                    self.recv.flush_pads();
                    EventChange(MessageEvent::VoiceTerm(lc), Idle)
                },
                Some(Err(err)) => {
                    self.recv.resync();
                    EventChange(MessageEvent::Error(err), Idle)
                },
                None => NoChange,
            },
            DecodeTSBK(ref mut dec) => match dec.feed(dibit) {
                Some(Ok(tsbk)) => {
                    if tsbk.is_tail() {
                        self.recv.flush_pads();
                    }

                    Event(MessageEvent::TrunkingControl(tsbk))
                },
                Some(Err(err)) => {
                    self.recv.resync();
                    EventChange(MessageEvent::Error(err), Idle)
                },
                None => NoChange,
            },
            Idle => NoChange,
        };

        match self.state {
            DecodeHeader(ref mut head) => self.stats.merge(head),
            DecodeLCFrameGroup(ref mut fg) => self.stats.merge(fg),
            DecodeCCFrameGroup(ref mut fg) => self.stats.merge(fg),
            DecodeLCTerminator(ref mut term) => self.stats.merge(term),
            DecodeTSBK(ref mut tsbk) => self.stats.merge(tsbk),
            Idle => {},
        }

        next
    }
}

impl HasStats for MessageReceiver {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}
