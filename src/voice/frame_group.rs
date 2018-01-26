//! Receive voice frame groups, known as LDU1 and LDU2 in the standard.
//!
//! Each frame group contains 9 voice frames, a low-speed data word, and an "extra"
//! packet: either link control (LC) or crypto control (CC).

use std;

use collect_slice::CollectSlice;

use bits::{Hexbit, HexbitBytes, Dibit};
use coding::{cyclic, hamming, reed_solomon};
use error::{P25Error, Result};
use stats::{Stats, HasStats};
use voice::frame::VoiceFrame;
use voice::{control, crypto};

use buffer::{
    Buffer,
    VoiceDataFragStorage,
    VoiceExtraStorage,
    VoiceExtraWordStorage,
    VoiceFrameStorage,
};

use consts::{
    CRYPTO_CONTROL_BYTES,
    EXTRA_HEXBITS,
    EXTRA_PIECE_DIBITS,
    LINK_CONTROL_BYTES,
};

use error::P25Error::*;
use self::State::*;
use self::StateChange::*;

/// Receiver for Link Control (LC) frame group.
pub type VoiceLCFrameGroupReceiver = FrameGroupReceiver<LinkControlExtra>;
/// Receiver for Crypto Control (CC) frame group.
pub type VoiceCCFrameGroupReceiver = FrameGroupReceiver<CryptoControlExtra>;

/// Internal state of the frame group receiver.
enum State {
    /// Decoding a voice frame.
    DecodeVoiceFrame(VoiceFrameReceiver),
    /// Decoding an "extra".
    DecodeExtra,
    /// Decoding a low-speed data fragment.
    DecodeDataFragment(DataFragmentReceiver),
    /// Finished decoding the frame group.
    Done,
}

impl State {
    /// Decode the upcoming symbols as a voice frame.
    pub fn decode_voice_frame() -> State {
        DecodeVoiceFrame(VoiceFrameReceiver::new())
    }

    /// Decode the upcoming symbols as a low-speed data fragment.
    pub fn decode_data_frag() -> State {
        DecodeDataFragment(DataFragmentReceiver::new())
    }
}

/// Action the state machine should take.
enum StateChange<E: Extra> {
    /// Do nothing.
    NoChange,
    /// Change to the enclosed state.
    Change(State),
    /// Change to the enclosed state and propagate an event.
    EventChange(FrameGroupEvent<E>, State),
    /// Propagate an error.
    Error(P25Error),
}

/// Events that can occur when receiving a frame group.
pub enum FrameGroupEvent<E: Extra> {
    /// Decoded a voice frame.
    VoiceFrame(VoiceFrame),
    /// Decoded an "extra" packet.
    Extra(E::Fields),
    /// Decoded a 16-bit fragment of the low-speed data word.
    DataFragment(u32),
}

/// State machine that receives the various pieces that make up a frame group.
pub struct FrameGroupReceiver<E: Extra> {
    /// Current state.
    state: State,
    /// Receiver for inner extra packet. This is persisted across state changes because
    /// each extra packet is split into 6 chunks and spread throughout the frame group.
    extra: ExtraReceiver<E>,
    /// The current frame position within the frame group.
    frame: usize,
    stats: Stats,
}

impl<E: Extra> FrameGroupReceiver<E> {
    /// Create a new `FrameGroupReceiver` in the initial state.
    pub fn new() -> FrameGroupReceiver<E> {
        FrameGroupReceiver {
            state: State::decode_voice_frame(),
            extra: ExtraReceiver::new(),
            frame: 0,
            stats: Stats::default(),
        }
    }

    /// Whether the full frame group has been received.
    pub fn done(&self) -> bool {
        if let Done = self.state { true } else { false }
    }

    /// Determine what action to take based on the given symbol.
    fn handle(&mut self, dibit: Dibit) -> StateChange<E> {
        let next = match self.state {
            DecodeVoiceFrame(ref mut decoder) => match decoder.feed(dibit) {
                Some(Ok(vf)) => {
                    self.frame += 1;

                    EventChange(FrameGroupEvent::VoiceFrame(vf), match self.frame {
                        1 => State::decode_voice_frame(),
                        2...7 => DecodeExtra,
                        8 => State::decode_data_frag(),
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
            DecodeDataFragment(ref mut dec) => match dec.feed(dibit) {
                Some(Ok(data)) => EventChange(FrameGroupEvent::DataFragment(data),
                                              State::decode_voice_frame()),
                Some(Err(err)) => Error(err),
                None => NoChange,
            },
            _ => unreachable!(),
        };

        match self.state {
            DecodeVoiceFrame(ref mut vf) => self.stats.merge(vf),
            DecodeExtra => self.stats.merge(&mut self.extra),
            DecodeDataFragment(ref mut df) => self.stats.merge(df),
            Done => {},
        }

        next
    }

    /// Feed in a baseband symbol, possibly producing an event. Return `Some(Ok(event))`
    /// if a nominal event occurred, `Some(Err(err))` if an error occurred, and `None` in
    /// the case of no event.
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

impl<E: Extra> HasStats for FrameGroupReceiver<E> {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}

/// An "extra" information packet carried along in a frame group.
pub trait Extra {
    /// Base decoder for the packet.
    type Fields;

    /// Decode the inner Reed Soloman code.
    fn decode_rs<'a>(buf: &'a mut [Hexbit; EXTRA_HEXBITS], s: &mut Stats)
        -> Result<&'a [Hexbit]>;
    /// Transform the given hexbits into a base packet decoder.
    fn decode_extra(buf: &[Hexbit]) -> Self::Fields;
}

/// Link control frame group extra.
pub struct LinkControlExtra;

impl Extra for LinkControlExtra {
    type Fields = control::LinkControlFields;

    fn decode_rs<'a>(buf: &'a mut [Hexbit; EXTRA_HEXBITS], s: &mut Stats)
        -> Result<&'a [Hexbit]>
    {
        reed_solomon::short::decode(buf).map(|(data, err)| {
            s.rs_short.record_fixes(err);
            data
        }).ok_or(RsShortUnrecoverable)
    }

    fn decode_extra(buf: &[Hexbit]) -> Self::Fields {
        let mut bytes = [0; LINK_CONTROL_BYTES];
        HexbitBytes::new(buf.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        control::LinkControlFields::new(bytes)
    }
}

/// Crypto control frame group extra.
pub struct CryptoControlExtra;

impl Extra for CryptoControlExtra {
    type Fields = crypto::CryptoControlFields;

    fn decode_rs<'a>(buf: &'a mut [Hexbit; EXTRA_HEXBITS], s: &mut Stats)
        -> Result<&'a [Hexbit]>
    {
        reed_solomon::medium::decode(buf).map(|(data, err)| {
            s.rs_med.record_fixes(err);
            data
        }).ok_or(RsMediumUnrecoverable)
    }

    fn decode_extra(buf: &[Hexbit]) -> Self::Fields {
        let mut bytes = [0; CRYPTO_CONTROL_BYTES];
        HexbitBytes::new(buf.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        crypto::CryptoControlFields::new(bytes)
    }
}

/// Receives and decodes an IMBE voice frame.
struct VoiceFrameReceiver {
    /// Current buffered dibits.
    dibits: Buffer<VoiceFrameStorage>,
    stats: Stats,
}

impl VoiceFrameReceiver {
    /// Create a new `VoiceFrameReceiver` in the initial state.
    pub fn new() -> VoiceFrameReceiver {
        VoiceFrameReceiver {
            dibits: Buffer::new(VoiceFrameStorage::new()),
            stats: Stats::default(),
        }
    }

    /// Feed in a baseband symbol, possibly resulting in a decoded voice frame. Return
    /// `Some(Ok(frame))` if a voice frame was successfully decoded, `Some(Err(err))` if
    /// an error occurred, and `None` in the case of no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<VoiceFrame>> {
        // HACK: work around borrow checker.
        let stats = &mut self.stats;

        self.dibits.feed(dibit).map(|buf| VoiceFrame::new(buf).map(|vf| {
            for idx in 0..4 {
                stats.golay_std.record_fixes(vf.errors[idx]);
            }

            for idx in 4..7 {
                stats.hamming_std.record_fixes(vf.errors[idx]);
            }

            vf
        }))
    }
}

impl HasStats for VoiceFrameReceiver {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}

/// Receives and decodes a frame group extra packet.
struct ExtraReceiver<E: Extra> {
    extra: std::marker::PhantomData<E>,
    /// Current buffered dibits for the current hexbit.
    dibits: Buffer<VoiceExtraWordStorage>,
    /// Current buffered hexbits.
    hexbits: Buffer<VoiceExtraStorage>,
    /// Number of dibits that have been received into the packet.
    dibit: usize,
    stats: Stats,
}

impl<E: Extra> ExtraReceiver<E> {
    /// Create a new `ExtraReceiver` in the initial state.
    pub fn new() -> ExtraReceiver<E> {
        ExtraReceiver {
            extra: std::marker::PhantomData,
            dibits: Buffer::new(VoiceExtraWordStorage::new()),
            hexbits: Buffer::new(VoiceExtraStorage::new()),
            dibit: 0,
            stats: Stats::default(),
        }
    }

    /// Whether the current piece of the packet is finished decoding.
    pub fn piece_done(&self) -> bool { self.dibit % EXTRA_PIECE_DIBITS == 0 }

    /// Feed in a baseband symbol, possibly producing a decoded packet. Return
    /// `Some(Ok(pkt))` if the packet was successfully decoded, `Some(Err(err))` if an
    /// error occurred, and `None` in the case of no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<E::Fields>> {
        self.dibit += 1;

        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u16,
            None => return None,
        };

        let bits = match hamming::shortened::decode(buf) {
            Some((data, err)) => {
                self.stats.hamming_short.record_fixes(err);
                data
            },
            // Let the following RS code attempt to fix these errors.
            None => 0,
        };

        let hexbits = match self.hexbits.feed(Hexbit::new(bits)) {
            Some(buf) => buf,
            None => return None,
        };

        Some(E::decode_rs(hexbits, &mut self.stats).map(|data| {
            E::decode_extra(data)
        }))
    }
}

impl<E: Extra> HasStats for ExtraReceiver<E> {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}

/// Receives a 16-bit fragment of the 32-bit "low-speed data" word embedded in each frame
/// group.
struct DataFragmentReceiver {
    /// Current buffer of coded dibits.
    dibits: Buffer<VoiceDataFragStorage>,
    /// Which byte is being received.
    byte: u8,
    /// Current decoded fragment.
    data: u32,
    stats: Stats,
}

impl DataFragmentReceiver {
    /// Create a new `DataFragmentReceiver` in the initial state.
    pub fn new() -> DataFragmentReceiver {
        DataFragmentReceiver {
            dibits: Buffer::new(VoiceDataFragStorage::new()),
            byte: 0,
            data: 0,
            stats: Stats::default(),
        }
    }

    /// Feed in a baseband symbol, possibly producing a decoded data fragment. Return
    /// `Some(Ok(frag))` if a fragment was successfully received, `Some(Err(err))` if an
    /// error occurred, and `None` in the case of no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<u32>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf as u16,
            None => return None,
        };

        let bits = match cyclic::decode(buf) {
            Some((data, err)) => {
                self.stats.cyclic.record_fixes(err);
                data
            },
            None => return Some(Err(CyclicUnrecoverable)),
        };

        self.byte += 1;

        self.data <<= 8;
        self.data |= bits as u32;

        if self.byte == 2 {
            Some(Ok(self.data))
        } else {
            None
        }
    }
}

impl HasStats for DataFragmentReceiver {
    fn stats(&mut self) -> &mut Stats { &mut self.stats }
}
