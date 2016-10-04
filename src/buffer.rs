use bits;
use data;
use message;
use voice;

/// Backing storage for a Buffer.
pub trait Storage {
    /// Type of item stored.
    type Input: Copy;
    /// Type of backing storage.
    type Buf;

    /// Number of items that can be stored.
    fn size() -> usize;
    /// Get the storage buffer.
    fn buf(&mut self) -> &mut Self::Buf;
    /// Add an item to the buffer at the given position.
    fn add(&mut self, item: Self::Input, pos: usize);
    /// Reset buffer to "empty" state.
    fn reset(&mut self);
}

macro_rules! storage_type {
    ($name:ident, [$input:ty; $size:expr]) => {
        pub struct $name([$input; $size]);

        impl $name {
            pub fn new() -> Self { $name([Default::default(); $size]) }
        }

        impl Storage for $name {
            type Input = $input;
            type Buf = [$input; $size];

            fn size() -> usize { $size }
            fn buf(&mut self) -> &mut Self::Buf { &mut self.0 }
            fn add(&mut self, item: Self::Input, pos: usize) { self.0[pos] = item; }
            // No need to reset because the buffer won't be seen in a non-full state.
            fn reset(&mut self) {}
        }
    };
}

/// Create a storage buffer for buffers smaller than 32 dibits.
macro_rules! small_storage_type {
    ($name:ident, $size:expr) => {
        pub struct $name(u64);

        impl $name {
            pub fn new() -> Self {
                assert!(Self::size() <= 32);
                $name(0)
            }
        }

        impl Storage for $name {
            type Input = bits::Dibit;
            type Buf = u64;

            fn size() -> usize { $size }
            fn buf(&mut self) -> &mut u64 { &mut self.0 }

            fn add(&mut self, item: Self::Input, _: usize) {
                self.0 <<= 2;
                self.0 |= item.bits() as u64;
            }

            fn reset(&mut self) { self.0 = 0; }
        }
    };
}

/// Stores hexbits that make up a voice header packet.
storage_type!(VoiceHeaderStorage, [bits::Hexbit; voice::consts::HEADER_HEXBITS]);
/// Stores dibits that make up a voice frame packet.
storage_type!(VoiceFrameStorage, [bits::Dibit; voice::consts::FRAME_DIBITS]);
/// Stores hexbits that make up a voice extra packet.
storage_type!(VoiceExtraStorage, [bits::Hexbit; voice::consts::EXTRA_HEXBITS]);
/// Stores dibits that make up a data/TSBK payload packet.
storage_type!(DataPayloadStorage, [bits::Dibit; data::consts::CODING_DIBITS]);
/// Stores dibits that make up the NID word.
small_storage_type!(NIDStorage, message::consts::NID_DIBITS);
/// Stores dibits that make up each coded word in a voice extra component.
small_storage_type!(VoiceExtraWordStorage, voice::consts::EXTRA_WORD_DIBITS);
/// Stores dibits that make up a voice data fragment.
small_storage_type!(VoiceDataFragStorage, voice::consts::DATA_FRAG_DIBITS);
/// Stores dibits that make up a coded word in a voice header packet.
small_storage_type!(VoiceHeaderWordStorage, voice::consts::HEADER_WORD_DIBITS);
/// Stores dibits that make up a coded word in a voice LC terminator packet.
small_storage_type!(VoiceLCTermWordStorage, voice::consts::LC_TERM_WORD_DIBITS);

pub struct Buffer<S: Storage> {
    storage: S,
    pos: usize,
}

impl<S: Storage> Buffer<S> {
    pub fn new(storage: S) -> Buffer<S> {
        Buffer {
            storage: storage,
            pos: 0,
        }
    }

    /// Add the given item to the buffer and return the buffer if it's completed.
    pub fn feed(&mut self, item: S::Input) -> Option<&mut S::Buf> {
        if self.pos == 0 {
            self.storage.reset();
        }

        self.storage.add(item, self.pos);
        self.pos += 1;

        if self.pos == S::size() {
            self.pos = 0;
            Some(self.storage.buf())
        } else {
            None
        }
    }
}
