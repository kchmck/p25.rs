use std;

use collect_slice::CollectSlice;

use bits::{Hexbit, HexbitBytes, Dibit};
use buffer::{Buffer, VoiceExtraStorage, VoiceFrameStorage, VoiceExtraWordStorage,
             VoiceDataFragStorage};
use coding::{cyclic, hamming, reed_solomon};
use error::{P25Error, Result};

use consts::{
    EXTRA_HEXBITS,
    EXTRA_PIECE_DIBITS,
    LINK_CONTROL_BYTES,
    CRYPTO_CONTROL_BYTES,
};

use voice::{control, crypto};
use voice::frame::VoiceFrame;

use error::P25Error::*;
use self::State::*;
use self::StateChange::*;

pub type VoiceLCFrameGroupReceiver = FrameGroupReceiver<LinkControlExtra>;
pub type VoiceCCFrameGroupReceiver = FrameGroupReceiver<CryptoControlExtra>;

pub trait Extra {
    type Fields;

    fn decode_rs(buf: &mut [Hexbit; EXTRA_HEXBITS]) -> Option<(&[Hexbit], usize)>;
    fn decode_extra(buf: &[Hexbit]) -> Self::Fields;
}

pub struct LinkControlExtra;

impl Extra for LinkControlExtra {
    type Fields = control::LinkControlFields;

    fn decode_rs(buf: &mut [Hexbit; EXTRA_HEXBITS]) -> Option<(&[Hexbit], usize)> {
        reed_solomon::short::decode(buf)
    }

    fn decode_extra(buf: &[Hexbit]) -> Self::Fields {
        let mut bytes = [0; LINK_CONTROL_BYTES];
        HexbitBytes::new(buf.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        control::LinkControlFields::new(bytes)
    }
}

pub struct CryptoControlExtra;

impl Extra for CryptoControlExtra {
    type Fields = crypto::CryptoControlFields;

    fn decode_rs(buf: &mut [Hexbit; EXTRA_HEXBITS]) -> Option<(&[Hexbit], usize)> {
        reed_solomon::medium::decode(buf)
    }

    fn decode_extra(buf: &[Hexbit]) -> Self::Fields {
        let mut bytes = [0; CRYPTO_CONTROL_BYTES];
        HexbitBytes::new(buf.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        crypto::CryptoControlFields::new(bytes)
    }
}

enum State {
    DecodeVoiceFrame(VoiceFrameReceiver),
    DecodeExtra,
    DecodeDataFragment,
    Done,
}

impl State {
    pub fn decode_voice_frame() -> State {
        DecodeVoiceFrame(VoiceFrameReceiver::new())
    }
}

enum StateChange<E: Extra> {
    NoChange,
    Change(State),
    EventChange(FrameGroupEvent<E>, State),
    Error(P25Error),
}

pub enum FrameGroupEvent<E: Extra> {
    VoiceFrame(VoiceFrame),
    Extra(E::Fields),
    DataFragment(u32),
}

pub struct FrameGroupReceiver<E: Extra> {
    state: State,
    extra: ExtraReceiver<E>,
    frag: DataFragmentReceiver,
    frame: usize,
}

impl<E: Extra> FrameGroupReceiver<E> {
    pub fn new() -> FrameGroupReceiver<E> {
        FrameGroupReceiver {
            state: State::decode_voice_frame(),
            extra: ExtraReceiver::new(),
            frag: DataFragmentReceiver::new(),
            frame: 0,
        }
    }

    pub fn done(&self) -> bool {
        if let Done = self.state { true } else { false }
    }

    fn handle(&mut self, dibit: Dibit) -> StateChange<E> {
        match self.state {
            DecodeVoiceFrame(ref mut decoder) => match decoder.feed(dibit) {
                Some(Ok(vf)) => {
                    self.frame += 1;

                    EventChange(FrameGroupEvent::VoiceFrame(vf), match self.frame {
                        1 => State::decode_voice_frame(),
                        2...7 => DecodeExtra,
                        8 => DecodeDataFragment,
                        9 => Done,
                        _ => unreachable!(),
                    })
                },
                Some(Err(e)) => Error(e),
                None => NoChange,
            },
            DecodeExtra => match self.extra.feed(dibit) {
                Some(Ok(extra)) => EventChange(FrameGroupEvent::Extra(extra),
                                               State::decode_voice_frame()),
                Some(Err(err)) => Error(err),
                None => if self.extra.piece_done() {
                    Change(State::decode_voice_frame())
                } else {
                    NoChange
                }
            },
            DecodeDataFragment => match self.frag.feed(dibit) {
                Some(Ok(data)) => EventChange(FrameGroupEvent::DataFragment(data),
                                              State::decode_voice_frame()),
                Some(Err(err)) => Error(err),
                None => NoChange,
            },
            _ => unreachable!(),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<FrameGroupEvent<E>>> {
        match self.handle(dibit) {
            EventChange(event, next) => {
                self.state = next;
                Some(Ok(event))
            },
            Change(state) => {
                self.state = state;
                None
            },
            Error(e) => Some(Err(e)),
            NoChange => None,
        }
    }
}

struct VoiceFrameReceiver {
    dibits: Buffer<VoiceFrameStorage>,
}

impl VoiceFrameReceiver {
    pub fn new() -> VoiceFrameReceiver {
        VoiceFrameReceiver {
            dibits: Buffer::new(VoiceFrameStorage::new()),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<VoiceFrame>> {
        match self.dibits.feed(dibit) {
            Some(buf) => Some(VoiceFrame::new(buf)),
            None => None,
        }
    }
}

struct ExtraReceiver<E: Extra> {
    extra: std::marker::PhantomData<E>,
    dibits: Buffer<VoiceExtraWordStorage>,
    hexbits: Buffer<VoiceExtraStorage>,
    dibit: usize,
}

impl<E: Extra> ExtraReceiver<E> {
    pub fn new() -> ExtraReceiver<E> {
        ExtraReceiver {
            extra: std::marker::PhantomData,
            dibits: Buffer::new(VoiceExtraWordStorage::new()),
            hexbits: Buffer::new(VoiceExtraStorage::new()),
            dibit: 0,
        }
    }

    pub fn piece_done(&self) -> bool { self.dibit % EXTRA_PIECE_DIBITS == 0 }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<E::Fields>> {
        self.dibit += 1;

        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u16,
            None => return None,
        };

        let bits = match hamming::shortened::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(HammingUnrecoverable)),
        };

        let hexbits = match self.hexbits.feed(Hexbit::new(bits)) {
            Some(buf) => buf,
            None => return None,
        };

        let data = match E::decode_rs(hexbits) {
            Some((data, err)) => data,
            None => return Some(Err(ReedSolomonUnrecoverable)),
        };

        Some(Ok(E::decode_extra(data)))
    }
}

struct DataFragmentReceiver {
    dibits: Buffer<VoiceDataFragStorage>,
    dibit: usize,
    data: u32,
}

impl DataFragmentReceiver {
    pub fn new() -> DataFragmentReceiver {
        DataFragmentReceiver {
            dibits: Buffer::new(VoiceDataFragStorage::new()),
            dibit: 0,
            data: 0,
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<u32>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u16,
            None => return None,
        };

        let bits = match cyclic::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(CyclicUnrecoverable)),
        };

        self.dibit += 1;

        self.data <<= 8;
        self.data |= bits as u32;

        if self.dibit == 2 {
            Some(Ok(self.data))
        } else {
            None
        }
    }
}
