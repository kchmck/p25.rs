//! Generate data blocks from a slice of bytes.

use std;
use std::ops::Range;

use data::crc;
use data::params::*;

/// Payload blocks for a confirmed data packet.
pub type ConfirmedPayload<'a> = Payload<'a, ConfirmedParams>;

/// Payload blocks for an unconfirmed data packet.
pub type UnconfirmedPayload<'a> = Payload<'a, UnconfirmedParams>;

/// Wraps a buffer of bytes, splitting them over payload blocks.
struct Payload<'a, P: PacketParams> {
    params: std::marker::PhantomData<P>,
    /// Data to split into blocks.
    data: &'a [u8],
}

impl<'a, P: PacketParams> Payload<'a, P> {
    /// Construct a new `Payload` over the given data bytes.
    pub fn new(data: &'a [u8]) -> Payload<'a, P> {
        assert!(data.len() <= P::packet_bytes());

        Payload {
            params: std::marker::PhantomData,
            data: data,
        }
    }

    /// Total number of blocks in the payload.
    pub fn blocks(&self) -> usize { P::blocks(self.data.len()) }

    /// Number of pad bytes in the payload.
    pub fn pads(&self) -> usize { P::pads(self.data.len()) }

    /// Construct an iterator over the normal blocks for the payload.
    pub fn iter(&self) -> PayloadIter<'a, P> {
        PayloadIter::new(self.data)
    }

    /// Get the tail block of the payload.
    pub fn tail(&self) -> TailBlock<'a, P> {
        // Clamp to index just past the end, so we get an empty slice in the case of no
        // data and just pads.
        let start = std::cmp::min(P::full_blocks(self.data.len()) * P::block_bytes(),
                                  self.data.len());
        let checksum = self.checksum();

        TailBlock::new(&self.data[start..], checksum)
    }

    /// Calculate the packet checksum over all data and pads.
    fn checksum(&self) -> u32 {
        crc::CRC32::new()
            .feed_bytes(self.data.iter().cloned())
            .feed_bytes((0..self.pads()).map(|_| 0))
            .finish() as u32
    }
}

/// Iterator over the normal (non-tail) blocks in a payload.
struct PayloadIter<'a, P: PacketParams> {
    params: std::marker::PhantomData<P>,
    /// Data to split into blocks (not all of it may be used).
    data: &'a [u8],
    /// Current block into the payload
    block: Range<usize>,
}

impl<'a, P: PacketParams> PayloadIter<'a, P> {
    /// Construct a new `PayloadIter` from the given data bytes.
    pub fn new(data: &'a [u8]) -> PayloadIter<'a, P> {
        PayloadIter {
            params: std::marker::PhantomData,
            block: 0..P::full_blocks(data.len()),
            data: data,
        }
    }
}

impl<'a, P: PacketParams> Iterator for PayloadIter<'a, P> {
    type Item = PayloadBlock<'a, P>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = match self.block.next() {
            Some(b) => b * P::block_bytes(),
            None => return None,
        };

        let stop = std::cmp::min(start + P::block_bytes(), self.data.len());

        Some(PayloadBlock::new(&self.data[start..stop]))
    }
}

/// Normal payload block.
struct PayloadBlock<'a, P: PacketParams> {
    params: std::marker::PhantomData<P>,
    /// Data part of the block.
    data: &'a [u8],
}

impl<'a, P: PacketParams> PayloadBlock<'a, P> {
    /// Construct a new `PayloadBlock` from the given data bytes.
    pub fn new(data: &'a [u8]) -> PayloadBlock<'a, P> {
        assert!(data.len() <= P::block_bytes());

        PayloadBlock {
            params: std::marker::PhantomData,
            data: data,
        }
    }

    /// Get the data and pad bytes that make up the block, in that order.
    pub fn build(&self) -> (&'a [u8], Range<usize>) {
        (self.data, 0..P::block_bytes() - self.data.len())
    }
}

/// Tail payload block, which has the packet checksum.
struct TailBlock<'a, P: PacketParams> {
    params: std::marker::PhantomData<P>,
    /// Data part of the block.
    data: &'a [u8],
    /// Packet checksum over all data and pad bytes.
    checksum: u32,
}

impl<'a, P: PacketParams> TailBlock<'a, P> {
    /// Construct a new `TailBlock` from the given data bytes and packet checksum.
    pub fn new(data: &'a [u8], checksum: u32) -> TailBlock<'a, P> {
        assert!(data.len() <= P::tail_bytes());

        TailBlock {
            params: std::marker::PhantomData,
            data: data,
            checksum: checksum,
        }
    }

    /// Get the data, pad, and checksum bytes that make up the block, in that order.
    pub fn build(&self) -> (&'a [u8], Range<usize>, [u8; 4]) {
        (self.data, 0..P::tail_bytes() - self.data.len(), self.checksum())
    }

    /// Convert the checksum to a byte array.
    fn checksum(&self) -> [u8; 4] {
        [
            (self.checksum >> 24) as u8,
            (self.checksum >> 16) as u8,
            (self.checksum >> 8) as u8,
            self.checksum as u8
        ]
    }
}

/// Additional fields used in confirmed data packet blocks.
pub struct ConfirmedBlockHeader {
    /// 7-bit serial number, used for retransmissions.
    serial_number: u8,
    /// 9-bit checksum over data and pad bytes in the block
    checksum: u16,
}

impl ConfirmedBlockHeader {
    /// Construct a new `ConfirmedBlockHeader` from the given 7-bit serial number, and use
    /// the given data and pads to calculate the checksum.
    pub fn new(serial_number: u8, data: &[u8], pads: Range<usize>) -> ConfirmedBlockHeader {
        assert!(serial_number >> 7 == 0);

        ConfirmedBlockHeader {
            serial_number: serial_number,
            checksum: Self::checksum(serial_number, data, pads),
        }
    }

    /// Get the header field.
    pub fn build(&self) -> [u8; 2] {
        [
            // Combine the serial number and MSB of the checksum.
            self.serial_number << 1 | (self.checksum >> 8) as u8,
            // Strip off the MSB.
            self.checksum as u8,
        ]
    }

    /// Calculate the block checksum.
    fn checksum(sn: u8, data: &[u8], pads: Range<usize>) -> u16 {
        crc::CRC9::new()
            .feed_bits(sn, 7)
            .feed_bytes(data.iter().cloned().chain(pads.map(|_| 0)))
            .finish() as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use data::params::*;
    use super::Payload;

    #[test]
    fn test_iter_nopad() {
        struct TestParams;

        impl PacketParams for TestParams {
            fn block_bytes() -> usize { 2 }
            fn tail_bytes() -> usize { 1 }
        }

        let bytes = [1, 2, 3, 4, 5];
        let b = Payload::<TestParams>::new(&bytes);

        assert_eq!(b.blocks(), 3);
        assert_eq!(b.pads(), 0);

        {
            let mut iter = b.iter();
            let (data, pads) = iter.next().unwrap().build();
            assert_eq!(data, &[1, 2]);
            assert_eq!(pads.len(), 0);
            let block = iter.next().unwrap();
            let (data, pads) = block.build();
            assert_eq!(data, &[3, 4]);
            assert_eq!(pads.len(), 0);
            assert!(iter.next().is_none());
        }

        {
            let (data, pads, _) = b.tail().build();
            assert_eq!(data, &[5]);
            assert_eq!(pads.len(), 0);
        }
    }

    #[test]
    fn test_iter_pad() {
        struct TestParams;

        impl PacketParams for TestParams {
            fn block_bytes() -> usize { 3 }
            fn tail_bytes() -> usize { 1 }
        }

        let bytes = [1, 2, 3, 4, 5];
        let b = Payload::<TestParams>::new(&bytes);

        assert_eq!(b.blocks(), 3);
        assert_eq!(b.pads(), 2);

        {
            let mut iter = b.iter();
            let (data, pads) = iter.next().unwrap().build();
            assert_eq!(data, &[1, 2, 3]);
            assert_eq!(pads.count(), 0);
            let (data, pads) = iter.next().unwrap().build();
            assert_eq!(data, &[4, 5]);
            assert_eq!(pads.count(), 1);
            assert!(iter.next().is_none());
        }

        {
            let (data, pads, _) = b.tail().build();
            assert_eq!(data.len(), 0);
            assert_eq!(pads.count(), 1);
        }
    }

    #[test]
    fn test_confirmed_payload() {
        let bytes = [
            0xFF, 0xF0, 0x0F, 0x00,
            0xFF, 0xFF, 0x0F, 0x00,
            0xFF, 0xF0, 0x0F, 0x00,
            0xFF, 0xFF, 0x0F, 0x00,
            0xFF, 0xF0, 0x0F, 0x00,
        ];

        let p = ConfirmedPayload::new(&bytes);

        assert_eq!(p.blocks(), 2);
        assert_eq!(p.pads(), 8);

        let mut iter = p.iter();

        {
            let (data, pads) = iter.next().unwrap().build();
            let header = ConfirmedBlockHeader::new(0b1100110, data, pads.clone()).build();

            assert_eq!(header, [
                0b11001100,
                0b01100101,
            ]);

            assert_eq!(data, &[
                0xFF, 0xF0, 0x0F, 0x00,
                0xFF, 0xFF, 0x0F, 0x00,
                0xFF, 0xF0, 0x0F, 0x00,
                0xFF, 0xFF, 0x0F, 0x00,
            ]);

            assert_eq!(pads.count(), 0);
        }

        assert!(iter.next().is_none());

        {
            let (data, pads, checksum) = p.tail().build();
            let header = ConfirmedBlockHeader::new(0b1100110, data, pads.clone()).build();

            assert_eq!(header, [
                0b11001101,
                0b01000000,
            ]);

            assert_eq!(data, &[
                0xFF, 0xF0, 0x0F, 0x00,
            ]);

            assert_eq!(pads.count(), 8);
            assert_eq!(checksum, [0x0C, 0x23, 0xD9, 0x14]);
        }
    }

    #[test]
    fn test_unconfirmed_payload() {
        let bytes = [
            0xFF, 0xF0, 0x0F, 0x00,
            0xFF, 0xFF, 0x0F, 0x00,
            0xFF, 0xF0, 0x0F, 0x00,

            0xFF, 0xFF, 0x0F, 0x00,
            0xFF, 0xF0, 0x0F, 0x00,
            0xFF, 0xF0,
        ];

        let p = UnconfirmedPayload::new(&bytes);

        assert_eq!(p.blocks(), 3);
        assert_eq!(p.pads(), 10);

        let mut iter = p.iter();

        {
            let (data, pads) = iter.next().unwrap().build();

            assert_eq!(data, &[
                0xFF, 0xF0, 0x0F, 0x00,
                0xFF, 0xFF, 0x0F, 0x00,
                0xFF, 0xF0, 0x0F, 0x00,
            ]);

            assert_eq!(pads.count(), 0);

            let (data, pads) = iter.next().unwrap().build();

            assert_eq!(data, &[
                0xFF, 0xFF, 0x0F, 0x00,
                0xFF, 0xF0, 0x0F, 0x00,
                0xFF, 0xF0,
            ]);

            assert_eq!(pads.count(), 2);
        }

        assert!(iter.next().is_none());

        {
            let (data, pads, checksum) = p.tail().build();

            assert_eq!(data, &[]);
            assert_eq!(pads.count(), 8);
            assert_eq!(checksum, [0x95, 0xe6, 0x14, 0xa2]);
        }
    }

    #[test]
    fn test_confirmed_checksums() {
        let bytes = [
            0x00, 0x00, 0x00, 0x00,
        ];

        let p = ConfirmedPayload::new(&bytes);

        assert_eq!(p.blocks(), 1);
        assert_eq!(p.pads(), 8);

        {
            let (data, pads, checksum) = p.tail().build();
            let header = ConfirmedBlockHeader::new(0, data, pads.clone()).build();

            assert_eq!(header, [
                0b00000001,
                0b11111111,
            ]);

            assert_eq!(checksum, [0xFF, 0xFF, 0xFF, 0xFF]);
        }
    }

    #[test]
    fn test_unconfirmed_checksum() {
        let bytes = [
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,

            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let p = UnconfirmedPayload::new(&bytes);

        assert_eq!(p.blocks(), 3);
        assert_eq!(p.pads(), 10);

        {
            let (data, pads, checksum) = p.tail().build();

            assert_eq!(data, &[]);
            assert_eq!(pads.count(), 8);
            assert_eq!(checksum, [0xFF, 0xFF, 0xFF, 0xFF]);
        }
    }
}
