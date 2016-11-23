//! Receive Trunking Signalling Block (TSBK) packets and decode the various TSBK payloads.

use collect_slice::CollectSlice;

use bits::{Dibit, DibitBytes};
use buffer::{Buffer, DataPayloadStorage};
use coding::trellis;
use consts::{TSBK_DIBITS, TSBK_BYTES};
use data::{crc, interleave};
use error::{Result, P25Error};
use util::{slice_u16, slice_u24};

use trunking::fields::{Channel, TalkGroup, ServiceOptions, RegResponse};

/// State machine for receiving a TSBK packet.
///
/// The state machine consumes dibit symbols and performs the following steps:
///
/// 1. Buffer dibits until a full packet's worth are available
/// 2. Descramble symbols using the same deinterleaver as data packets
/// 3. Decode 1/2-rate convolutional code and attempt to correct any errors
/// 4. Group dibits into a buffer of bytes for further interpretation
pub struct TSBKReceiver {
    /// Current buffered dibits.
    dibits: Buffer<DataPayloadStorage>,
}

impl TSBKReceiver {
    /// Create a new `TSBKReceiver` in the initial state.
    pub fn new() -> TSBKReceiver {
        TSBKReceiver {
            dibits: Buffer::new(DataPayloadStorage::new()),
        }
    }

    /// Feed in a baseband symbol, possibly producing a complete TSBK packet. Return
    /// `Some(Ok(pkt))` if a packet was successfully received, `Some(Err(err))` if an
    /// error occurred, and `None` in the case of no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<TSBKFields>> {
        let (count, dibits) = {
            let buf = match self.dibits.feed(dibit) {
                Some(buf) => buf,
                None => return None,
            };

            let mut dibits = [Dibit::default(); TSBK_DIBITS];
            let count = trellis::DibitDecoder::new(interleave::Deinterleaver::new(buf))
                .filter_map(|x| x.ok())
                .collect_slice(&mut dibits[..]);

            (count, dibits)
        };

        if count != dibits.len() {
            return Some(Err(P25Error::ViterbiUnrecoverable));
        }

        let mut bytes = [0; TSBK_BYTES];
        DibitBytes::new(dibits.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        Some(Ok(TSBKFields::new(bytes)))
    }
}

/// Type of a TSBK payload.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TSBKOpcode {
    GroupVoiceGrant,
    GroupVoiceUpdate,
    GroupVoiceUpdateExplicit,
    UnitVoiceGrant,
    UnitCallRequest,
    UnitVoiceUpdate,
    PhoneGrant,
    PhoneCallRequest,
    UnitDataGrant,
    GroupDataGrant,
    GroupDataUpdate,
    GroupDataUpdateExplicit,
    UnitStatusUpdate,
    UnitStatusQuery,
    UnitShortMessage,
    UnitMonitor,
    UnitCallAlert,
    AckResponse,
    QueuedResponse,
    ExtendedFunctionResponse,
    DenyResponse,
    GroupAffiliationResponse,
    GroupAffiliationQuery,
    LocRegResponse,
    UnitRegResponse,
    UnitRegCommand,
    UnitAuthCommand,
    UnitDeregisterAck,
    RoamingAddrCommand,
    RoamingAddrUpdate,
    SystemServiceBroadcast,
    AltControlChannel,
    RFSSStatusBroadcast,
    NetworkStatusBroadcast,
    AdjacentSite,
    ChannelParamsUpdate,
    ProtectionParamBroadcast,
    ProtectionParamUpdate,
    Reserved,
}

impl TSBKOpcode {
    /// Try to parse an opcode from the given 6 bits.
    pub fn from_bits(bits: u8) -> Option<TSBKOpcode> {
        use self::TSBKOpcode::*;

        assert!(bits >> 6 == 0);

        match bits {
            0b000000 => Some(GroupVoiceGrant),
            0b000001 => Some(Reserved),
            0b000010 => Some(GroupVoiceUpdate),
            0b000011 => Some(GroupVoiceUpdateExplicit),
            0b000100 => Some(UnitVoiceGrant),
            0b000101 => Some(UnitCallRequest),
            0b000110 => Some(UnitVoiceUpdate),
            0b000111 => Some(Reserved),

            0b001000 => Some(PhoneGrant),
            0b001001 => Some(Reserved),
            0b001010 => Some(PhoneCallRequest),
            0b001011...0b001111 => Some(Reserved),

            0b010000 => Some(UnitDataGrant),
            0b010001 => Some(GroupDataGrant),
            0b010010 => Some(GroupDataUpdate),
            0b010011 => Some(GroupDataUpdateExplicit),
            0b010100...0b010111 => Some(Reserved),

            0b011000 => Some(UnitStatusUpdate),
            0b011001 => Some(Reserved),
            0b011010 => Some(UnitStatusQuery),
            0b011011 => Some(Reserved),
            0b011100 => Some(UnitShortMessage),
            0b011101 => Some(UnitMonitor),
            0b011110 => Some(Reserved),
            0b011111 => Some(UnitCallAlert),
            0b100000 => Some(AckResponse),
            0b100001 => Some(QueuedResponse),
            0b100010 => Some(Reserved),
            0b100011 => Some(Reserved),
            0b100100 => Some(ExtendedFunctionResponse),
            0b100101 => Some(Reserved),
            0b100110 => Some(Reserved),
            0b100111 => Some(DenyResponse),

            0b101000 => Some(GroupAffiliationResponse),
            0b101001 => Some(Reserved),
            0b101010 => Some(GroupAffiliationQuery),
            0b101011 => Some(LocRegResponse),
            0b101100 => Some(UnitRegResponse),
            0b101101 => Some(UnitRegCommand),
            0b101110 => Some(UnitAuthCommand),
            0b101111 => Some(UnitDeregisterAck),

            0b110000...0b110101 => Some(Reserved),
            0b110110 => Some(RoamingAddrCommand),
            0b110111 => Some(RoamingAddrUpdate),

            0b111000 => Some(SystemServiceBroadcast),
            0b111001 => Some(AltControlChannel),
            0b111010 => Some(RFSSStatusBroadcast),
            0b111011 => Some(NetworkStatusBroadcast),
            0b111100 => Some(AdjacentSite),
            0b111101 => Some(ChannelParamsUpdate),
            0b111110 => Some(ProtectionParamBroadcast),
            0b111111 => Some(ProtectionParamUpdate),

            _ => None,
        }
    }
}

/// Buffer of bytes that represents a TSBK packet.
pub type Buf = [u8; TSBK_BYTES];

/// A Trunking Signalling Block packet.
#[derive(Copy, Clone)]
pub struct TSBKFields(Buf);

impl TSBKFields {
    /// Interpret the given bytes as a TSBK packet.
    pub fn new(buf: Buf) -> TSBKFields { TSBKFields(buf) }

    /// Whether this packet is the last one in the TSBK group.
    pub fn is_tail(&self) -> bool { self.0[0] >> 7 == 1 }
    /// Whether the packet is encrypted.
    pub fn protected(&self) -> bool { self.0[0] >> 6 & 1 == 1 }
    /// Type of data contained in the payload.
    pub fn opcode(&self) -> Option<TSBKOpcode> { TSBKOpcode::from_bits(self.0[0] & 0x3F) }
    /// Manufacturer ID, which determines if the packet is standardized.
    pub fn mfg(&self) -> u8 { self.0[1] }
    /// Transmitted CRC.
    pub fn crc(&self) -> u16 { slice_u16(&self.0[10..]) }

    /// Calculate 16-bit CRC over bytes in packet.
    pub fn calc_crc(&self) -> u16 {
        crc::CRC16::new()
            .feed_bytes((&self.0[..10]).iter().cloned())
            .finish() as u16
    }

    /// Verify if the calculated CRC matches the transmitted one.
    pub fn crc_valid(&self) -> bool {
        self.crc() == self.calc_crc()
    }

    /// Bytes that make up the payload of the packet.
    pub fn payload(&self) -> &[u8] { &self.0[2...9] }
}

/// Response given to an attempted user registration.
pub struct UnitRegResponse(Buf);

impl UnitRegResponse {
    /// Create a new `UnitRegResponse` decoder from the base TSBK decoder.
    pub fn new(tsbk: TSBKFields) -> Self { UnitRegResponse(tsbk.0) }

    /// System response to user registration request.
    pub fn response(&self) -> RegResponse {
        RegResponse::from_bits((self.0[2] >> 4) & 0b11)
    }

    /// System ID within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[2...3]) & 0xFFF }
    /// Address of originating unit, which uniquely identifies the unit within the System.
    pub fn src_id(&self) -> u32 { slice_u24(&self.0[4...6]) }
    /// ID of originating unit which, along with the WACN and System ID, uniquely
    /// identifies the unit.
    pub fn src_addr(&self) -> u32 { slice_u24(&self.0[7...9]) }
}

pub struct GroupVoiceGrant(Buf);

impl GroupVoiceGrant {
    pub fn new(tsbk: TSBKFields) -> Self { GroupVoiceGrant(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[5..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitVoiceGrant(Buf);

impl UnitVoiceGrant {
    pub fn new(tsbk: TSBKFields) -> Self { UnitVoiceGrant(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitCallRequest(Buf);

impl UnitCallRequest {
    pub fn new(tsbk: TSBKFields) -> Self { UnitCallRequest(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_id(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitVoiceUpdate(Buf);

impl UnitVoiceUpdate {
    pub fn new(tsbk: TSBKFields) -> Self { UnitVoiceUpdate(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct PhoneGrant(Buf);

impl PhoneGrant {
    pub fn new(tsbk: TSBKFields) -> Self { PhoneGrant(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[3..]) }
    pub fn call_timer(&self) -> u16 { slice_u16(&self.0[5..]) }
    pub fn unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitDataGrant(Buf);

impl UnitDataGrant {
    pub fn new(tsbk: TSBKFields) -> Self { UnitDataGrant(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

#[cfg(test)]
mod test {
    use super::*;
    use trunking::fields::*;

    #[test]
    fn test_tsbk_fields() {
        let t = TSBKFields::new([
            0b10111001,
            0b00000001,
            0b11110000,
            0b00001111,
            0b10101010,
            0b01010101,
            0b00000000,
            0b11111111,
            0b11001100,
            0b00110011,
            0b11010111,
            0b11010111,
        ]);

        assert!(t.is_tail());
        assert!(!t.protected());
        assert_eq!(t.opcode(), Some(TSBKOpcode::AltControlChannel));
        assert_eq!(t.mfg(), 0b00000001);
        assert_eq!(t.crc(), 0b1101011111010111);
        assert_eq!(t.calc_crc(), 0b0111010000111100);
        assert!(!t.crc_valid());
        assert_eq!(t.payload(), &[
            0b11110000,
            0b00001111,
            0b10101010,
            0b01010101,
            0b00000000,
            0b11111111,
            0b11001100,
            0b00110011,
        ]);
    }

    #[test]
    fn test_adjacent_site() {
        let t = TSBKFields::new([
            0b00111100,
            0b00000000,
            0b11001100,
            0b11011111,
            0b00111100,
            0b10101010,
            0b01010101,
            0b00110110,
            0b01111110,
            0b01010001,
            0b00000000,
            0b00000000,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::AdjacentSite));
        let a = AdjacentSite::new(t.payload());

        assert_eq!(a.area(), 0b11001100);
        assert!(a.opts().conventional());
        assert!(a.opts().failing());
        assert!(!a.opts().current());
        assert!(a.opts().networked());
        assert_eq!(a.system(), 0b111100111100);
        assert_eq!(a.rfss(), 0b10101010);
        assert_eq!(a.site(), 0b01010101);
        assert_eq!(a.channel().id(), 0b0011);
        assert_eq!(a.channel().number(), 0b011001111110);
        let s = a.services();
        assert!(s.is_composite());
        assert!(!s.has_updates());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_channel_params_update() {
        let t = TSBKFields::new([
            0b00111101,
            0b00000000,
            0b0110_0011,
            0b00100_010,
            0b110100_00,
            0b00110010,
            0b00001010,
            0b00100101,
            0b00010000,
            0b10100010,
            0b11111111,
            0b11111111,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::ChannelParamsUpdate));
        let p = ChannelParamsUpdate::new(t.payload());

        assert_eq!(p.id(), 0b0110);
        assert_eq!(p.params().bandwidth, 12_500);
        assert_eq!(p.params().rx_freq(0b1001), 851_062_500);
    }

    #[test]
    fn test_group_voice_update() {
        let t = TSBKFields::new([
            0b00000010,
            0b00000000,
            0b01101111,
            0b01010101,
            0b11111111,
            0b11111111,
            0b10011010,
            0b10101010,
            0b00110011,
            0b11001100,
            0b00000000,
            0b00000000,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::GroupVoiceUpdate));
        let u = GroupVoiceUpdate::new(t.payload()).updates();

        assert_eq!(u[0].0.id(), 0b0110);
        assert_eq!(u[0].0.number(), 0b111101010101);
        assert_eq!(u[0].1, TalkGroup::Everbody);
        assert_eq!(u[1].0.id(), 0b1001);
        assert_eq!(u[1].0.number(), 0b101010101010);
        assert_eq!(u[1].1, TalkGroup::Other(0b0011001111001100));
    }

    #[test]
    fn test_alt_control_channel() {
        let t = TSBKFields::new([
            0b00111001,
            0b00000000,
            0b11100011,
            0b01010101,
            0b10110110,
            0b10101111,
            0b01010001,
            0b11101010,
            0b10101010,
            0b10101110,
            0b00000000,
            0b11111111,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::AltControlChannel));
        let a = AltControlChannel::new(t.payload());
        assert_eq!(a.rfss(), 0b11100011);
        assert_eq!(a.site(), 0b01010101);
        let c = a.alts();
        assert_eq!(c[0].0.id(), 0b1011);
        assert_eq!(c[0].0.number(), 0b011010101111);
        let s = c[0].1;
        assert!(s.is_composite());
        assert!(!s.has_updates());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
        assert_eq!(c[1].0.id(), 0b1110);
        assert_eq!(c[1].0.number(), 0b101010101010);
        let s = c[1].1;
        assert!(!s.is_composite());
        assert!(s.has_updates());
        assert!(s.is_backup());
        assert!(!s.has_data());
        assert!(s.has_voice());
        assert!(!s.has_registration());
        assert!(s.has_auth());
    }

    #[test]
    fn test_rfss_status_broadcast() {
        let t = TSBKFields::new([
            0b00111010,
            0b00000000,
            0b11001100,
            0b00010000,
            0b10101010,
            0b11100111,
            0b00011000,
            0b11010101,
            0b01110011,
            0b01010001,
            0b00000000,
            0b00000000,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::RFSSStatusBroadcast));
        let a = RFSSStatusBroadcast::new(t.payload());
        assert_eq!(a.area(), 0b11001100);
        assert!(a.networked());
        assert_eq!(a.system(), 0b000010101010);
        assert_eq!(a.rfss(), 0b11100111);
        assert_eq!(a.site(), 0b00011000);
        assert_eq!(a.channel().id(), 0b1101);
        assert_eq!(a.channel().number(), 0b010101110011);
        let s = a.services();
        assert!(s.is_composite());
        assert!(!s.has_updates());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_network_status_broadcast() {
        let t = TSBKFields::new([
            0b00111011,
            0b00000000,
            0b11001010,
            0b11111100,
            0b00101011,
            0b11001111,
            0b01011011,
            0b11011100,
            0b11100111,
            0b01010001,
            0b00000000,
            0b00000000,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::NetworkStatusBroadcast));
        let n = NetworkStatusBroadcast::new(t.payload());
        assert_eq!(n.area(), 0b11001010);
        assert_eq!(n.wacn(), 0b11111100001010111100);
        assert_eq!(n.system(), 0b111101011011);
        assert_eq!(n.channel().id(), 0b1101);
        assert_eq!(n.channel().number(), 0b110011100111);
        let s = n.services();
        assert!(s.is_composite());
        assert!(!s.has_updates());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_unit_reg_response() {
        let t = TSBKFields::new([
            0b00101100,
            0b00000000,
            0b11011010,
            0b11100111,
            0b10101010,
            0b01010101,
            0b00110011,
            0b11111000,
            0b00111111,
            0b11001100,
            0b00000000,
            0b00000000,
        ]);
        assert_eq!(t.opcode(), Some(TSBKOpcode::UnitRegResponse));
        let r = UnitRegResponse::new(t);
        assert_eq!(r.response(), RegResponse::Fail);
        assert_eq!(r.system(), 0b101011100111);
        assert_eq!(r.src_id(), 0b101010100101010100110011);
        assert_eq!(r.src_addr(), 0b111110000011111111001100);
    }
}
