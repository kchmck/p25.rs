//! State machine for buffering items until a buffer is full.

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

/// Create a storage buffer backed by a fixed array.
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

/// Implements a state machine that buffers items to a backing store and notifies the caller when
/// the buffer is full.
pub struct Buffer<S: Storage> {
    /// Backing storage.
    storage: S,
    /// Current number of buffered items.
    pos: usize,
}

impl<S: Storage> Buffer<S> {
    /// Create a new `Buffer` with the given backing storage.
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

#[cfg(test)]
mod test {
    use super::{Buffer, Storage};
    use bits;

    storage_type!(TestStorage, [u8; 5]);
    small_storage_type!(TestSmallStorage, 7);

    #[test]
    fn test_storage() {
        assert_eq!(TestStorage::size(), 5);
        let mut s = TestStorage::new();
        s.add(42, 0);
        assert_eq!(s.buf()[0], 42);
        s.add(37, 0);
        s.add(64, 4);
        assert_eq!(s.buf()[0], 37);
        assert_eq!(s.buf()[4], 64);
    }

    #[test]
    fn test_small_storage() {
        assert_eq!(TestSmallStorage::size(), 7);
        let mut s = TestSmallStorage::new();
        assert_eq!(s.buf(), &0);
        s.add(bits::Dibit::new(0b11), 0);
        s.add(bits::Dibit::new(0b01), 1);
        // Doesn't take position into account.
        s.add(bits::Dibit::new(0b10), 0);
        assert_eq!(s.buf(), &0b110110);
    }

    #[test]
    fn test_buffer_storage() {
        let mut b = Buffer::new(TestStorage::new());
        assert_eq!(b.feed(13), None);
        assert_eq!(b.feed(17), None);
        assert_eq!(b.feed(23), None);
        assert_eq!(b.feed(31), None);
        assert_eq!(b.feed(37), Some(&mut [13, 17, 23, 31, 37]));
        assert_eq!(b.feed(42), None);
        assert_eq!(b.feed(52), None);
        assert_eq!(b.feed(62), None);
        assert_eq!(b.feed(72), None);
        assert_eq!(b.feed(82), Some(&mut [42, 52, 62, 72, 82]));
        assert_eq!(b.feed(92), None);
    }

    #[test]
    fn test_buffer_small() {
        let mut b = Buffer::new(TestSmallStorage::new());
        assert_eq!(b.feed(bits::Dibit::new(0b11)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b01)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b01)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b00)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b11)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b10)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b01)), Some(&mut 0b11010100111001));
        assert_eq!(b.feed(bits::Dibit::new(0b10)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b11)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b11)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b11)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b00)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b00)), None);
        assert_eq!(b.feed(bits::Dibit::new(0b10)), Some(&mut 0b10111111000010));
        assert_eq!(b.feed(bits::Dibit::new(0b00)), None);
    }
}
