use consts::LINK_CONTROL_BYTES;
use trunking::fields::{TalkGroup, Channel, SystemServices, ServiceOptions};
use util::{slice_u16, slice_u24};

pub type Buf = [u8; LINK_CONTROL_BYTES];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LinkControlOpcode {
    GroupVoiceTraffic,
    GroupVoiceChannel,
    UnitVoiceTraffic,
    GroupVoiceChannelExplicit,
    UnitVoiceRequest,
    PhoneVoiceTraffic,
    PhoneVoiceRequest,
    CallTermination,
    GroupAffiliationQuery,
    UnitRegistrationRequest,
    UnitAuthenticationRequst,
    UnitStatusRequest,
    SystemServiceBroadcast,
    SecondaryControlBroadcast,
    AdjacentSiteBroadcast,
    RadioStatusBroadcast,
    NetworkStatusBroadcast,
    UnitStatus,
    UnitMessage,
    UnitCallAlert,
    ExtendedFunction,
    ChannelParamsUpdate,
    ProtectionParamBroadcast,
    AltControlBroadcastExplicit,
    AdjacentSiteBroadcastExplicit,
    ChannelIdentifierExplicit,
    RadioStatusExplicit,
    NetworkStatusExplicit,
}

impl LinkControlOpcode {
    pub fn from_bits(bits: u8) -> Option<LinkControlOpcode> {
        use self::LinkControlOpcode::*;

        match bits {
            0b000000 => Some(GroupVoiceTraffic),
            0b000010 => Some(GroupVoiceChannel),
            0b000011 => Some(UnitVoiceTraffic),
            0b000100 => Some(GroupVoiceChannelExplicit),
            0b000101 => Some(UnitVoiceRequest),
            0b000110 => Some(PhoneVoiceTraffic),
            0b000111 => Some(PhoneVoiceRequest),
            0b001111 => Some(CallTermination),
            0b010000 => Some(GroupAffiliationQuery),
            0b010001 => Some(UnitRegistrationRequest),
            0b010010 => Some(UnitAuthenticationRequst),
            0b010011 => Some(UnitStatusRequest),
            0b100000 => Some(SystemServiceBroadcast),
            0b100001 => Some(SecondaryControlBroadcast),
            0b100010 => Some(AdjacentSiteBroadcast),
            0b100011 => Some(RadioStatusBroadcast),
            0b100100 => Some(NetworkStatusBroadcast),
            0b010100 => Some(UnitStatus),
            0b010101 => Some(UnitMessage),
            0b010110 => Some(UnitCallAlert),
            0b010111 => Some(ExtendedFunction),
            0b011000 => Some(ChannelParamsUpdate),
            0b100101 => Some(ProtectionParamBroadcast),
            0b100110 => Some(AltControlBroadcastExplicit),
            0b100111 => Some(AdjacentSiteBroadcastExplicit),
            0b011001 => Some(ChannelIdentifierExplicit),
            0b101000 => Some(RadioStatusExplicit),
            0b101001 => Some(NetworkStatusExplicit),
            _ => None,
        }
    }
}

pub struct LinkControlFields(Buf);

impl LinkControlFields {
    pub fn new(buf: Buf) -> Self { LinkControlFields(buf) }

    pub fn protected(&self) -> bool { self.0[0] >> 7 == 1 }

    pub fn opcode(&self) -> Option<LinkControlOpcode> {
        LinkControlOpcode::from_bits(self.0[0] & 0x3F)
    }
}

pub struct GroupVoiceTraffic(Buf);

impl GroupVoiceTraffic {
    pub fn new(lc: LinkControlFields) -> Self { GroupVoiceTraffic(lc.0) }

    pub fn mfg(&self) -> u8 { self.0[1] }
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[6..]) }
}

pub struct GroupVoiceChannel(Buf);

impl GroupVoiceChannel {
    pub fn new(lc: LinkControlFields) -> Self { GroupVoiceChannel(lc.0) }

    pub fn channel_a(&self) -> Channel { Channel::new(&self.0[1..]) }
    pub fn talk_group_a(&self) -> TalkGroup { TalkGroup::new(&self.0[3..]) }

    pub fn channel_b(&self) -> Channel { Channel::new(&self.0[5..]) }
    pub fn talk_group_b(&self) -> TalkGroup { TalkGroup::new(&self.0[7..]) }
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

pub struct AdjacentSiteBroadcast(Buf);

impl AdjacentSiteBroadcast {
    pub fn new(lc: LinkControlFields) -> Self { AdjacentSiteBroadcast(lc.0) }
    pub fn area(&self) -> u8 { self.0[1] }
    pub fn system(&self) -> u16 { slice_u16(&self.0[2..]) & 0xFFF }
    pub fn rfss(&self) -> u8 { self.0[4] }
    pub fn site(&self) -> u8 { self.0[5] }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[6..]) }
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[8]) }
}

#[cfg(test)]
mod test {
    use super::*;
    use trunking::fields::{TalkGroup};

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
