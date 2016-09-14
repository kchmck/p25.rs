use bits;
use data;
use voice;

/// Backing storage for a Buffer.
pub trait Storage {
    /// Type of item stored.
    type Input: Copy;
    /// Type of backing storage.
    type Buf;

    /// Number of items that can be stored.
    fn size(&self) -> usize;
    /// Get the storage buffer.
    fn buf(&mut self) -> &mut Self::Buf;
    /// Add an item to the buffer at the given position.
    fn add(&mut self, item: Self::Input, pos: usize);
    /// Reset buffer to "empty" state.
    fn reset(&mut self);
}

macro_rules! storage_type {
    ($name: ident, [$input: ty; $size: expr]) => {
        pub struct $name([$input; $size]);

        impl $name {
            pub fn new() -> Self { $name([Default::default(); $size]) }
        }

        impl Storage for $name {
            type Input = $input;
            type Buf = [$input; $size];

            fn size(&self) -> usize { $size }
            fn buf(&mut self) -> &mut Self::Buf { &mut self.0 }
            fn add(&mut self, item: Self::Input, pos: usize) { self.0[pos] = item; }
            // No need to reset because the buffer won't be seen in a non-full state.
            fn reset(&mut self) {}
        }
    };
}

storage_type!(VoiceHeaderStorage, [bits::Hexbit; voice::consts::HEADER_HEXBITS]);
storage_type!(VoiceFrameStorage, [bits::Dibit; voice::consts::FRAME_DIBITS]);
storage_type!(VoiceExtraStorage, [bits::Hexbit; voice::consts::EXTRA_HEXBITS]);
storage_type!(DataPayloadStorage, [bits::Dibit; data::consts::CODING_DIBITS]);

pub struct DibitStorage {
    buf: u64,
    size: usize,
}

impl DibitStorage {
    pub fn new(size: usize) -> DibitStorage {
        assert!(size <= 32);

        DibitStorage {
            buf: 0,
            size: size,
        }
    }
}

impl Storage for DibitStorage {
    type Input = bits::Dibit;
    type Buf = u64;

    fn size(&self) -> usize { self.size }
    fn buf(&mut self) -> &mut u64 { &mut self.buf }

    fn add(&mut self, item: Self::Input, _: usize) {
        self.buf <<= 2;
        self.buf |= item.bits() as u64;
    }

    fn reset(&mut self) { self.buf = 0; }
}

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

    pub fn reset(&mut self) { self.pos = 0; }

    /// Add the given item to the buffer and return the buffer if it's completed.
    pub fn feed(&mut self, item: S::Input) -> Option<&mut S::Buf> {
        if self.pos == 0 {
            self.storage.reset();
        }

        self.storage.add(item, self.pos);
        self.pos += 1;

        if self.pos == self.storage.size() {
            self.reset();
            Some(self.storage.buf())
        } else {
            None
        }
    }
}
