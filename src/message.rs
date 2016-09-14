use error::P25Error;
use nid::NetworkID;
use receiver::DataUnitReceiver;
use status::StreamSymbol;
use trunking::tsbk::{TSBKFields, TSBKReceiver};
use voice::control::LinkControlFields;
use voice::crypto::CryptoControlFields;
use voice::frame::VoiceFrame;
use voice::header::VoiceHeaderFields;

use voice::{
    FrameGroupEvent,
    VoiceCCFrameGroupReceiver,
    VoiceHeaderReceiver,
    VoiceLCFrameGroupReceiver,
    VoiceLCTerminatorReceiver,
};

pub trait MessageHandler {
    fn handle_error(&mut self, recv: &mut DataUnitReceiver, err: P25Error);
    fn handle_nid(&mut self, recv: &mut DataUnitReceiver, nid: NetworkID);
    fn handle_header(&mut self, recv: &mut DataUnitReceiver, header: VoiceHeaderFields);
    fn handle_frame(&mut self, recv: &mut DataUnitReceiver, frame: VoiceFrame);
    fn handle_lc(&mut self, recv: &mut DataUnitReceiver, lc: LinkControlFields);
    fn handle_cc(&mut self, recv: &mut DataUnitReceiver, cc: CryptoControlFields);
    fn handle_data_frag(&mut self, recv: &mut DataUnitReceiver, data: u32);
    fn handle_tsbk(&mut self, recv: &mut DataUnitReceiver, tsbk: TSBKFields);
    fn handle_term(&mut self, recv: &mut DataUnitReceiver);
}

enum State {
    Idle,
    DecodeHeader(VoiceHeaderReceiver),
    DecodeLCFrameGroup(VoiceLCFrameGroupReceiver),
    DecodeCCFrameGroup(VoiceCCFrameGroupReceiver),
    DecodeLCTerminator(VoiceLCTerminatorReceiver),
    DecodeTSBK(TSBKReceiver),
}

pub struct MessageReceiver {
    recv: DataUnitReceiver,
    state: State,
}

impl MessageReceiver {
    pub fn new() -> MessageReceiver {
        MessageReceiver {
            recv: DataUnitReceiver::new(),
            state: State::Idle,
        }
    }

    pub fn feed<H: MessageHandler>(&mut self, s: f32, handler: &mut H) {
        use self::State::*;
        use nid::DataUnit::*;
        use receiver::ReceiverEvent;

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
                handler.handle_nid(&mut self.recv, nid);

                self.state = match nid.data_unit() {
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
                    FrameGroupEvent::DataFragment(data) => handler.handle_data_frag(&mut self.recv, data),
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
