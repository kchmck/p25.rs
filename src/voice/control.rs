//! Decode Link Control (LC) packets and payloads.

use consts::LINK_CONTROL_BYTES;
use util::{slice_u16, slice_u24};

use trunking::fields::{TalkGroup, ServiceOptions};

/// Buffer of bytes that represents a link control packet.
pub type Buf = [u8; LINK_CONTROL_BYTES];

/// Type of a link control payload.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LinkControlOpcode {
    GroupVoiceTraffic,
    GroupVoiceUpdate,
    UnitVoiceTraffic,
    GroupVoiceUpdateExplicit,
    UnitCallRequest,
    PhoneTraffic,
    PhoneAlert,
    CallTermination,
    GroupAffiliationQuery,
    UnitRegistrationRequest,
    UnitAuthenticationRequst,
    UnitStatusRequest,
    SystemServiceBroadcast,
    AltControlChannel,
    AdjacentSite,
    RfssStatusBroadcast,
    NetworkStatusBroadcast,
    UnitStatusUpdate,
    UnitShortMessage,
    UnitCallAlert,
    ExtendedFunction,
    ChannelParamsUpdate,
    ProtectionParamBroadcast,
    AltControlChannelExplicit,
    AdjacentSiteExplicit,
    ChannelParamsExplicit,
    RfssStatusExplicit,
    NetworkStatusExplicit,
}

impl LinkControlOpcode {
    /// Try to parse an opcode from the given 6 bits.
    pub fn from_bits(bits: u8) -> Option<LinkControlOpcode> {
        use self::LinkControlOpcode::*;

        assert!(bits >> 6 == 0);

        match bits {
            0b000000 => Some(GroupVoiceTraffic),
            0b000010 => Some(GroupVoiceUpdate),
            0b000011 => Some(UnitVoiceTraffic),
            0b000100 => Some(GroupVoiceUpdateExplicit),
            0b000101 => Some(UnitCallRequest),
            0b000110 => Some(PhoneTraffic),
            0b000111 => Some(PhoneAlert),
            0b001111 => Some(CallTermination),
            0b010000 => Some(GroupAffiliationQuery),
            0b010001 => Some(UnitRegistrationRequest),
            0b010010 => Some(UnitAuthenticationRequst),
            0b010011 => Some(UnitStatusRequest),
            0b100000 => Some(SystemServiceBroadcast),
            0b100001 => Some(AltControlChannel),
            0b100010 => Some(AdjacentSite),
            0b100011 => Some(RfssStatusBroadcast),
            0b100100 => Some(NetworkStatusBroadcast),
            0b010100 => Some(UnitStatusUpdate),
            0b010101 => Some(UnitShortMessage),
            0b010110 => Some(UnitCallAlert),
            0b010111 => Some(ExtendedFunction),
            0b011000 => Some(ChannelParamsUpdate),
            0b100101 => Some(ProtectionParamBroadcast),
            0b100110 => Some(AltControlChannelExplicit),
            0b100111 => Some(AdjacentSiteExplicit),
            0b011001 => Some(ChannelParamsExplicit),
            0b101000 => Some(RfssStatusExplicit),
            0b101001 => Some(NetworkStatusExplicit),
            _ => None,
        }
    }
}

/// Base link control decoder, common to all packets.
#[derive(Copy, Clone)]
pub struct LinkControlFields(Buf);

impl LinkControlFields {
    /// Interpret the given bytes as a link control packet.
    pub fn new(buf: Buf) -> Self { LinkControlFields(buf) }

    /// Whether the packet is encrypted.
    pub fn protected(&self) -> bool { self.0[0] >> 7 == 1 }

    /// Type of data contained in the payload.
    pub fn opcode(&self) -> Option<LinkControlOpcode> {
        LinkControlOpcode::from_bits(self.0[0] & 0x3F)
    }

    /// Bytes that make up the payload.
    pub fn payload(&self) -> &[u8] { &self.0[1..=8] }
}

/// Identity of unit transmitting on the current talkgroup traffic channel.
pub struct GroupVoiceTraffic(Buf);

impl GroupVoiceTraffic {
    /// Create a new `GroupVoiceTraffic` from the base LC decoder.
    pub fn new(lc: LinkControlFields) -> Self { GroupVoiceTraffic(lc.0) }

    /// Manufacturer ID of current packet.
    pub fn mfg(&self) -> u8 { self.0[1] }
    /// Service options provided by current traffic channel.
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    /// Current resident talkgroup of traffic channel.
    pub fn talkgroup(&self) -> TalkGroup { TalkGroup::new(&self.0[4..]) }
    /// Address of user currently transmitting.
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

/// Identity of units transmitting on current unit-to-unit traffic channel.
pub struct UnitVoiceTraffic(Buf);

impl UnitVoiceTraffic {
    /// Create a new `UnitVoiceTraffic` from the base LC decoder.
    pub fn new(lc: LinkControlFields) -> Self { UnitVoiceTraffic(lc.0) }

    /// Manufacturer ID of current packet.
    pub fn mfg(&self) -> u8 { self.0[1] }
    /// Service options provided by current traffic channel.
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    /// Destination user address for current transmission.
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[3..]) }
    /// Source user address for current transmission.
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

/// Identity of unit participating in current phone call.
pub struct PhoneTraffic(Buf);

impl PhoneTraffic {
    /// Create a new `PhoneTraffic` decoder from the base LC decoder.
    pub fn new(lc: LinkControlFields) -> Self { PhoneTraffic(lc.0) }

    /// Options requested/granted for the traffic channel.
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    /// Maximum amount of time (in units of 100ms) that the phone call can occupy the
    /// traffic channel.
    pub fn call_timer(&self) -> u16 { slice_u16(&self.0[4..=5]) }
    /// Unit participating in call.
    pub fn unit(&self) -> u32 { slice_u24(&self.0[6..=8]) }
}

#[cfg(test)]
mod test {
    use super::*;
    use trunking::fields::*;

    #[test]
    fn test_lc() {
        let lc = LinkControlFields::new([
            0b00000000,
            0b00000000,
            0b10110101, 0b00000000,
            0b00000000, 0b00000001,
            0xDE, 0xAD, 0xBE,
        ]);

        assert_eq!(lc.opcode(), Some(LinkControlOpcode::GroupVoiceTraffic));
        assert_eq!(lc.protected(), false);

        assert_eq!(lc.payload(), &[
            0b00000000,
            0b10110101, 0b00000000,
            0b00000000, 0b00000001,
            0xDE, 0xAD, 0xBE,
        ]);
    }

    #[test]
    fn test_adjacent_site() {
        let lc = LinkControlFields::new([
            0b10100010,
            0b11001100,
            0b00001111,
            0b01010101,
            0b11100011,
            0b00011000,
            0b11000001,
            0b11111111,
            0b01010001,
        ]);
        assert_eq!(lc.opcode(), Some(LinkControlOpcode::AdjacentSite));
        let a = AdjacentSite::new(lc.payload());

        assert_eq!(a.area(), 0b11001100);
        assert_eq!(a.system(), 0b111101010101);
        assert_eq!(a.rfss(), 0b11100011);
        assert_eq!(a.site(), 0b00011000);
        assert_eq!(a.channel().id(), 0b1100);
        assert_eq!(a.channel().number(), 0b000111111111);
        let s = a.services();
        assert!(s.is_composite());
        assert!(!s.updates_only());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_group_voice_traffic() {
        let lc = LinkControlFields::new([
            0b00000000,
            0b00000000,
            0b10110101, 0b00000000,
            0b00000000, 0b00000001,
            0xDE, 0xAD, 0xBE,
        ]);
        assert_eq!(lc.opcode(), Some(LinkControlOpcode::GroupVoiceTraffic));
        let dec = GroupVoiceTraffic::new(lc);
        let opts = dec.opts();

        assert_eq!(dec.mfg(), 0);
        assert_eq!(dec.talkgroup(), TalkGroup::Default);
        assert_eq!(dec.src_unit(), 0xDEADBE);

        assert_eq!(opts.emergency(), true);
        assert_eq!(opts.protected(), false);
        assert_eq!(opts.full_duplex(), true);
        assert_eq!(opts.packet_switched(), true);
        assert_eq!(opts.prio(), 5);
    }

    #[test]
    fn test_channel_params_update() {
        let l = LinkControlFields::new([
            0b00011000,
            0b01100011,
            0b00100010,
            0b11010000,
            0b00110010,
            0b00001010,
            0b00100101,
            0b00010000,
            0b10100010,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::ChannelParamsUpdate));
        let p = ChannelParamsUpdate::new(l.payload());

        assert_eq!(p.id(), 0b0110);
        assert_eq!(p.params().bandwidth, 12_500);
        assert_eq!(p.params().rx_freq(0b1001), 851_062_500);
    }

    #[test]
    fn test_group_traffic_update() {
        let l = LinkControlFields::new([
            0b00000010,
            0b01101111,
            0b01010101,
            0b11111111,
            0b11111111,
            0b10011010,
            0b10101010,
            0b00110011,
            0b11001100,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::GroupVoiceUpdate));
        let u = GroupTrafficUpdate::new(l.payload()).updates();

        assert_eq!(u[0].0.id(), 0b0110);
        assert_eq!(u[0].0.number(), 0b111101010101);
        assert_eq!(u[0].1, TalkGroup::Everbody);
        assert_eq!(u[1].0.id(), 0b1001);
        assert_eq!(u[1].0.number(), 0b101010101010);
        assert_eq!(u[1].1, TalkGroup::Other(0b0011001111001100));
    }

    #[test]
    fn test_alt_control_channel() {
        let l = LinkControlFields::new([
            0b00100001,
            0b11100011,
            0b01010101,
            0b10110110,
            0b10101111,
            0b01010001,
            0b11101010,
            0b10101010,
            0b10101110,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::AltControlChannel));
        let a = AltControlChannel::new(l.payload());
        assert_eq!(a.rfss(), 0b11100011);
        assert_eq!(a.site(), 0b01010101);
        let c = a.alts();
        assert_eq!(c[0].0.id(), 0b1011);
        assert_eq!(c[0].0.number(), 0b011010101111);
        let s = c[0].1;
        assert!(s.is_composite());
        assert!(!s.updates_only());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
        assert_eq!(c[1].0.id(), 0b1110);
        assert_eq!(c[1].0.number(), 0b101010101010);
        let s = c[1].1;
        assert!(!s.is_composite());
        assert!(s.updates_only());
        assert!(s.is_backup());
        assert!(!s.has_data());
        assert!(s.has_voice());
        assert!(!s.has_registration());
        assert!(s.has_auth());
    }

    #[test]
    fn test_rfss_status_broadcast() {
        let l = LinkControlFields::new([
            0b00100011,
            0b11001100,
            0b00010000,
            0b10101010,
            0b11100111,
            0b00011000,
            0b11010101,
            0b01110011,
            0b01010001,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::RfssStatusBroadcast));
        let a = RfssStatusBroadcast::new(l.payload());
        assert_eq!(a.area(), 0b11001100);
        assert!(a.networked());
        assert_eq!(a.system(), 0b000010101010);
        assert_eq!(a.rfss(), 0b11100111);
        assert_eq!(a.site(), 0b00011000);
        assert_eq!(a.channel().id(), 0b1101);
        assert_eq!(a.channel().number(), 0b010101110011);
        let s = a.services();
        assert!(s.is_composite());
        assert!(!s.updates_only());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_network_status_broadcast() {
        let l = LinkControlFields::new([
            0b00100100,
            0b11001010,
            0b11111100,
            0b00101011,
            0b11001111,
            0b01011011,
            0b11011100,
            0b11100111,
            0b01010001,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::NetworkStatusBroadcast));
        let n = NetworkStatusBroadcast::new(l.payload());
        assert_eq!(n.area(), 0b11001010);
        assert_eq!(n.wacn(), 0b11111100001010111100);
        assert_eq!(n.system(), 0b111101011011);
        assert_eq!(n.channel().id(), 0b1101);
        assert_eq!(n.channel().number(), 0b110011100111);
        let s = n.services();
        assert!(s.is_composite());
        assert!(!s.updates_only());
        assert!(!s.is_backup());
        assert!(s.has_data());
        assert!(!s.has_voice());
        assert!(s.has_registration());
        assert!(!s.has_auth());
    }

    #[test]
    fn test_call_alert() {
        let l = LinkControlFields::new([
            0b00010110,
            0b11111111,
            0b11111111,
            0b01010101,
            0b10101010,
            0b11001100,
            0b00110011,
            0b11100111,
            0b00011000,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::UnitCallAlert));
        let c = UnitCallAlert::new(l.payload());
        assert_eq!(c.dest_unit(), 0b010101011010101011001100);
        assert_eq!(c.src_unit(), 0b001100111110011100011000);
    }

    #[test]
    fn test_call_request() {
        let l = LinkControlFields::new([
            0b00000101,
            0b01010101,
            0b11111111,
            0b00111001,
            0b11000110,
            0b01010101,
            0b11101010,
            0b00010101,
            0b11110000,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::UnitCallRequest));
        let r = UnitCallRequest::new(l.payload());
        let o = r.opts();
        assert!(!o.emergency());
        assert!(o.protected());
        assert!(!o.full_duplex());
        assert!(o.packet_switched());
        assert_eq!(o.prio(), 0b101);
        assert_eq!(r.dest_unit(), 0b001110011100011001010101);
        assert_eq!(r.src_unit(), 0b111010100001010111110000);
    }

    #[test]
    fn test_phone_alert() {
        let l = LinkControlFields::new([
            0b00000111,
            0b11110011,
            0b00111100,
            0b01011010,
            0b11100111,
            0b01101110,
            0b11111100,
            0b01111110,
            0b00111111,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::PhoneAlert));
        let a = PhoneAlert::new(l.payload());
        assert_eq!(a.digits(), &[
            0b11110011,
            0b00111100,
            0b01011010,
            0b11100111,
            0b01101110,
        ]);
        assert_eq!(a.dest_unit(), 0b111111000111111000111111);
    }

    #[test]
    fn test_phone_traffic() {
        let l = LinkControlFields::new([
            0b00000110,
            0b00000000,
            0b01010101,
            0b00000000,
            0b10000000,
            0b00000010,
            0b11110000,
            0b00110011,
            0b11100010,
        ]);
        assert_eq!(l.opcode(), Some(LinkControlOpcode::PhoneTraffic));
        let p = PhoneTraffic::new(l);
        let o = p.opts();
        assert!(!o.emergency());
        assert!(o.protected());
        assert!(!o.full_duplex());
        assert!(o.packet_switched());
        assert_eq!(o.prio(), 0b101);
        assert_eq!(p.call_timer(), 0b1000000000000010);
        assert_eq!(p.unit(), 0b111100000011001111100010);
    }
}
