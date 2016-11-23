use consts::LINK_CONTROL_BYTES;
use util::slice_u24;

use trunking::fields::{
    TalkGroup,
    ServiceOptions,
    ChannelUpdates,
    parse_updates,
};

pub type Buf = [u8; LINK_CONTROL_BYTES];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LinkControlOpcode {
    GroupVoiceTraffic,
    GroupVoiceUpdate,
    UnitVoiceTraffic,
    GroupVoiceUpdateExplicit,
    UnitVoiceRequest,
    PhoneVoiceTraffic,
    PhoneVoiceRequest,
    CallTermination,
    GroupAffiliationQuery,
    UnitRegistrationRequest,
    UnitAuthenticationRequst,
    UnitStatusRequest,
    SystemServiceBroadcast,
    AltControlChannel,
    AdjacentSite,
    RFSSStatusBroadcast,
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
    RFSSStatusExplicit,
    NetworkStatusExplicit,
}

impl LinkControlOpcode {
    pub fn from_bits(bits: u8) -> Option<LinkControlOpcode> {
        use self::LinkControlOpcode::*;

        match bits {
            0b000000 => Some(GroupVoiceTraffic),
            0b000010 => Some(GroupVoiceUpdate),
            0b000011 => Some(UnitVoiceTraffic),
            0b000100 => Some(GroupVoiceUpdateExplicit),
            0b000101 => Some(UnitVoiceRequest),
            0b000110 => Some(PhoneVoiceTraffic),
            0b000111 => Some(PhoneVoiceRequest),
            0b001111 => Some(CallTermination),
            0b010000 => Some(GroupAffiliationQuery),
            0b010001 => Some(UnitRegistrationRequest),
            0b010010 => Some(UnitAuthenticationRequst),
            0b010011 => Some(UnitStatusRequest),
            0b100000 => Some(SystemServiceBroadcast),
            0b100001 => Some(AltControlChannel),
            0b100010 => Some(AdjacentSite),
            0b100011 => Some(RFSSStatusBroadcast),
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
            0b101000 => Some(RFSSStatusExplicit),
            0b101001 => Some(NetworkStatusExplicit),
            _ => None,
        }
    }
}

/// Link Control information carried within voice packets.
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
    pub fn payload(&self) -> &[u8] { &self.0[1...8] }
}

pub struct GroupVoiceTraffic(Buf);

impl GroupVoiceTraffic {
    pub fn new(lc: LinkControlFields) -> Self { GroupVoiceTraffic(lc.0) }

    pub fn mfg(&self) -> u8 { self.0[1] }
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

pub struct GroupVoiceUpdate(Buf);

impl GroupVoiceUpdate {
    pub fn new(lc: LinkControlFields) -> Self { GroupVoiceUpdate(lc.0) }

    pub fn updates(&self) -> ChannelUpdates { parse_updates(&self.0[1...8]) }
}

pub struct UnitVoiceTraffic(Buf);

impl UnitVoiceTraffic {
    pub fn new(lc: LinkControlFields) -> Self { UnitVoiceTraffic(lc.0) }

    pub fn mfg(&self) -> u8 { self.0[1] }
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[3..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

pub struct CallTermination(Buf);

impl CallTermination {
    pub fn new(lc: LinkControlFields) -> Self { CallTermination(lc.0) }
    pub fn unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

#[cfg(test)]
mod test {
    use super::*;
    use trunking::fields::{TalkGroup, AdjacentSite};

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
            0b10100111,
            0b11001100,
            0b00001111,
            0b01010101,
            0b11100011,
            0b00011000,
            0b11000001,
            0b11111111,
            0b01010001,
        ]);
        let a = AdjacentSite::new(lc.payload());

        assert_eq!(a.area(), 0b11001100);
        assert_eq!(a.system(), 0b111101010101);
        assert_eq!(a.rfss(), 0b11100011);
        assert_eq!(a.site(), 0b00011000);
        assert_eq!(a.channel().id(), 0b1100);
        assert_eq!(a.channel().number(), 0b000111111111);
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
    fn test_group_voice_traffic() {
        let lc = LinkControlFields::new([
            0b00000000,
            0b00000000,
            0b10110101, 0b00000000,
            0b00000000, 0b00000001,
            0xDE, 0xAD, 0xBE,
        ]);

        let dec = GroupVoiceTraffic::new(lc);
        let opts = dec.opts();

        assert_eq!(dec.mfg(), 0);
        assert_eq!(dec.talk_group(), TalkGroup::Default);
        assert_eq!(dec.src_unit(), 0xDEADBE);

        assert_eq!(opts.emergency(), true);
        assert_eq!(opts.protected(), false);
        assert_eq!(opts.duplex(), true);
        assert_eq!(opts.packet_switched(), true);
        assert_eq!(opts.prio(), 5);
    }
}
