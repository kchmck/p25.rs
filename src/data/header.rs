//! Header generation for data packets.
//!
//! A header has several user-filled fields followed by a 16-bit checksum over those
//! fields.

use data::crc;
use data::values;

/// Packet header block for confirmed data packet.
pub type ConfirmedHeader = Header<ConfirmedFields>;

/// Packet header block for unconfirmed data packet.
pub type UnconfirmedHeader = Header<UnconfirmedFields>;

/// Write some bytes into a buffer.
pub trait BufWrite {
    fn write<'a, 'b, T: Iterator<Item = &'a mut u8>>(&self, buf: &'b mut T);
}

/// Field is only a single byte.
pub trait ByteField {
    fn byte(&self) -> u8;
}

/// Just write the single byte.
impl<B: ByteField> BufWrite for B {
    fn write<'a, 'b, T: Iterator<Item = &'a mut u8>>(&self, mut buf: &'b mut T) {
        *buf.next().unwrap() = self.byte();
    }
}

/// Preamble header field.
pub struct HeaderPreamble {
    /// Whether the packet requires confirmation.
    pub confirmed: bool,
    /// Whether the packet is an outbound message.
    pub outbound: bool,
    /// Packet type.
    pub format: values::DataPacket,
}

impl ByteField for HeaderPreamble {
    fn byte(&self) -> u8 {
        bool_to_bit(self.confirmed) << 6 | bool_to_bit(self.outbound) << 5 |
            self.format.to_bits()
    }
}

/// Preamble for confirmed data packet.
pub struct ConfirmedPreamble(HeaderPreamble);

impl ConfirmedPreamble {
    fn new(outbound: bool) -> ConfirmedPreamble {
        ConfirmedPreamble(HeaderPreamble {
            confirmed: true,
            outbound: outbound,
            format: values::DataPacket::ConfirmedPacket,
        })
    }

    pub fn outbound() -> Self { Self::new(true) }
    pub fn inbound() -> Self { Self::new(false) }
}

impl ByteField for ConfirmedPreamble {
    fn byte(&self) -> u8 { self.0.byte() }
}

/// Preamble for unconfirmed data packet.
pub struct UnconfirmedPreamble(HeaderPreamble);

impl UnconfirmedPreamble {
    fn new(outbound: bool) -> UnconfirmedPreamble {
        UnconfirmedPreamble(HeaderPreamble {
            confirmed: false,
            outbound: outbound,
            format: values::DataPacket::UnconfirmedPacket
        })
    }

    pub fn outbound() -> Self { Self::new(true) }
    pub fn inbound() -> Self { Self::new(false) }
}

impl ByteField for UnconfirmedPreamble {
    fn byte(&self) -> u8 { self.0.byte() }
}

/// Service access point (SAP) field.
pub struct ServiceAccessPoint(pub values::ServiceAccessPoint);

impl ByteField for ServiceAccessPoint {
    fn byte(&self) -> u8 {
        0b11000000 | self.0.to_bits()
    }
}

/// Manufacturer's ID field.
pub struct Manufacturer(pub u8);

impl ByteField for Manufacturer {
    fn byte(&self) -> u8 { self.0 }
}

/// Logical link ID field for addressing source or destination subscriber.
pub struct LogicalLink(pub u32);

impl BufWrite for LogicalLink {
    fn write<'a, 'b, T: Iterator<Item = &'a mut u8>>(&self, mut buf: &'b mut T) {
        assert!(self.0 >> 24 == 0);

        *buf.next().unwrap() = (self.0 >> 16) as u8;
        *buf.next().unwrap() = (self.0 >> 8) as u8;
        *buf.next().unwrap() = self.0 as u8;
    }
}

/// FMF and blocks-to-follow fields.
pub struct BlockCount {
    /// Whether the packet is "complete", not being partially retransmitted.
    pub full_pkt: bool,
    /// Number of data blocks in the packet.
    pub count: u8,
}

impl ByteField for BlockCount {
    fn byte(&self) -> u8 {
        assert!(self.count >> 7 == 0);
        bool_to_bit(self.full_pkt) << 7 | self.count
    }
}

/// Number of pad bytes at the end of the data.
pub struct PadCount(pub u8);

impl ByteField for PadCount {
    fn byte(&self) -> u8 {
        assert!(self.0 >> 5 == 0);
        self.0
    }
}

/// Syn, N(S), and FSNF fields.
pub struct Sequencing {
    /// Whether the receiver should resynchronize its sequence numbers using `pkt_seq` and
    /// `frag_seq`.
    pub resync: bool,
    /// Packet sequence number, used for ordering and duplicate removal.
    pub pkt_seq: u8,
    /// Fragment sequence number, used when data is split across multiple fragments.
    pub frag_seq: u8,
}

impl ByteField for Sequencing {
    fn byte(&self) -> u8 {
        assert!(self.pkt_seq >> 3 == 0);
        assert!(self.frag_seq >> 4 == 0);

        bool_to_bit(self.resync) << 7 | self.pkt_seq << 4 | self.frag_seq
    }
}

/// Byte offset into data payload where data header stops and data information begins.
pub struct DataOffset(pub u8);

impl ByteField for DataOffset {
    fn byte(&self) -> u8 {
        assert!(self.0 >> 6 == 0);
        self.0
    }
}

/// Header fields for confirmed packet.
pub struct ConfirmedFields {
    pub preamble: ConfirmedPreamble,
    pub sap: ServiceAccessPoint,
    pub mfg: Manufacturer,
    pub addr: LogicalLink,
    pub blocks: BlockCount,
    pub pads: PadCount,
    pub seq: Sequencing,
    pub data_offset: DataOffset,
}

impl BufWrite for ConfirmedFields {
    fn write<'a, 'b, T: Iterator<Item = &'a mut u8>>(&self, mut buf: &'b mut T) {
        self.preamble.write(buf);
        self.sap.write(buf);
        self.mfg.write(buf);
        self.addr.write(buf);
        self.blocks.write(buf);
        self.pads.write(buf);
        self.seq.write(buf);
        self.data_offset.write(buf);
    }
}

/// Header fields for unconfirmed packet.
pub struct UnconfirmedFields {
    pub preamble: UnconfirmedPreamble,
    pub sap: ServiceAccessPoint,
    pub mfg: Manufacturer,
    pub addr: LogicalLink,
    pub blocks: BlockCount,
    pub pads: PadCount,
    pub data_offset: DataOffset,
}

impl BufWrite for UnconfirmedFields {
    fn write<'a, 'b, T: Iterator<Item = &'a mut u8>>(&self, mut buf: &'b mut T) {
        self.preamble.write(buf);
        self.sap.write(buf);
        self.mfg.write(buf);
        self.addr.write(buf);
        self.blocks.write(buf);
        self.pads.write(buf);
        *buf.next().unwrap() = 0;
        self.data_offset.write(buf);
    }
}

/// Builds a checksummed header based on the given fields.
pub struct Header<F: BufWrite>(F);

impl<F: BufWrite> Header<F> {
    /// Construct a new `Header` with the given fields.
    pub fn new(fields: F) -> Header<F> {
        Header(fields)
    }

    /// Get the fields and checksum that make up the header, in that order.
    pub fn build(self) -> ([u8; 10], [u8; 2]) {
        let fields = self.fields();
        let checksum = self.checksum(&fields);

        (fields, checksum)
    }

    /// Build a byte buffer from the header fields.
    fn fields(&self) -> [u8; 10] {
        let mut buf = [0; 10];
        self.0.write(&mut buf.iter_mut());
        buf
    }

    /// Calculate the checksum of the header fields.
    fn checksum(&self, fields: &[u8]) -> [u8; 2] {
        assert!(fields.len() == 10);

        let checksum = crc::CRC16::new()
            .feed_bytes(fields.iter().cloned())
            .finish();

        [(checksum >> 8) as u8, checksum as u8]
    }
}

/// Convert the given Boolean to a single bit.
fn bool_to_bit(b: bool) -> u8 {
    if b { 1 } else { 0 }
}

#[cfg(test)]
mod test {
    use super::*;
    use data::values;

    #[test]
    fn test_preamble() {
        let p = ConfirmedPreamble::outbound();
        assert_eq!(p.byte(), 0b01110110);
        let p = ConfirmedPreamble::inbound();
        assert_eq!(p.byte(), 0b01010110);
    }

    #[test]
    fn test_sap() {
        let s = ServiceAccessPoint(values::ServiceAccessPoint::ExtendedAddressing);
        assert_eq!(s.byte(), 0b11011111);
    }

    #[test]
    fn test_mfg() {
        let m = Manufacturer(0b11011011);
        assert_eq!(m.byte(), 0b11011011);
    }

    #[test]
    fn test_ll() {
        let l = LogicalLink(0xABCDEF);
        let mut buf = [0; 3];
        l.write(&mut buf.iter_mut());
        assert_eq!(&buf, &[0xAB, 0xCD, 0xEF]);
    }

    #[test]
    fn test_bc() {
        let b = BlockCount {
            full_pkt: true,
            count: 127,
        };

        assert_eq!(b.byte(), 0b11111111);
    }

    #[test]
    fn test_pc() {
        let p = PadCount(12);
        assert_eq!(p.byte(), 0b00001100);
    }

    #[test]
    fn test_seq() {
        let s = Sequencing {
            resync: false,
            pkt_seq: 6,
            frag_seq: 10,
        };
        assert_eq!(s.byte(), 0b01101010);
    }

    #[test]
    fn test_do() {
        let d = DataOffset(11);
        assert_eq!(d.byte(), 0b00001011);
    }

    #[test]
    fn test_confirmed_fields() {
        let fields = ConfirmedFields {
            preamble: ConfirmedPreamble::outbound(),
            sap: ServiceAccessPoint(values::ServiceAccessPoint::Paging),
            mfg: Manufacturer(0x12),
            addr: LogicalLink(0x342134),
            blocks: BlockCount {
                full_pkt: true,
                count: 127,
            },
            pads: PadCount(3),
            seq: Sequencing {
                resync: false,
                pkt_seq: 5,
                frag_seq: 2,
            },
            data_offset: DataOffset(0),
        };

        let mut buf = [0; 10];
        fields.write(&mut buf.iter_mut());

        assert_eq!(&buf, &[
            0b01110110,
            0b11100110,
            0b00010010,
            0b00110100,
            0b00100001,
            0b00110100,
            0b11111111,
            0b00000011,
            0b01010010,
            0b00000000,
        ]);
    }

    #[test]
    fn test_confirmed_header() {
        let (fields, checksum) = ConfirmedHeader::new(ConfirmedFields {
            preamble: ConfirmedPreamble::outbound(),
            sap: ServiceAccessPoint(values::ServiceAccessPoint::PacketData),
            mfg: Manufacturer(0x12),
            addr: LogicalLink(0x342134),
            blocks: BlockCount {
                full_pkt: true,
                count: 127,
            },
            pads: PadCount(3),
            seq: Sequencing {
                resync: false,
                pkt_seq: 5,
                frag_seq: 2,
            },
            data_offset: DataOffset(0),
        }).build();

        assert_eq!(fields, [
            0b01110110,
            0b11000100,
            0b00010010,
            0b00110100,
            0b00100001,
            0b00110100,
            0b11111111,
            0b00000011,
            0b01010010,
            0b00000000,
        ]);

        assert_eq!(checksum, [
            0b10001010,
            0b01110010,
        ]);
    }

    #[test]
    #[should_panic]
    fn test_ll_validate() {
        let mut buf = [0; 4];
        let mut iter = buf.iter_mut();
        LogicalLink(0x20FFFFFF).write(&mut iter);
    }

    #[test]
    #[should_panic]
    fn test_bc_validate() {
        BlockCount {
            full_pkt: true,
            count: 0b11111111,
        }.byte();
    }

    #[test]
    #[should_panic]
    fn test_pc_validate() {
        PadCount(0b11111111).byte();
    }

    #[test]
    #[should_panic]
    fn test_seq_validate() {
        Sequencing {
            resync: true,
            pkt_seq: 0b11111111,
            frag_seq: 0b11111111,
        }.byte();
    }

    #[test]
    #[should_panic]
    fn test_do_validate() {
        DataOffset(0b11111111).byte();
    }
}
