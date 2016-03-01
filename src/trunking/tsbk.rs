use collect_slice::CollectSlice;

use bits::{Dibit, DibitBytes};
use buffer::{Buffer, DataPayloadStorage};
use coding::trellis;
use data::{crc, interleave};
use error::{Result, P25Error};

use trunking::decode::*;

pub struct TSBKDecoder {
    dibits: Buffer<DataPayloadStorage>,
}

impl TSBKDecoder {
    pub fn new() -> TSBKDecoder {
        TSBKDecoder {
            dibits: Buffer::new(DataPayloadStorage::new()),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<TSBK>> {
        let (count, dibits) = {
            let buf = match self.dibits.feed(dibit) {
                Some(buf) => buf,
                None => return None,
            };

            let mut dibits = [Dibit::default(); 48];
            let count = trellis::DibitDecoder::new(interleave::Deinterleaver::new(buf))
                .filter(|x| x.is_ok()).map(|x| x.unwrap())
                .collect_slice(&mut dibits[..]);

            (count, dibits)
        };

        self.dibits.reset();

        if count != dibits.len() {
            return Some(Err(P25Error::ViterbiUnrecoverable));
        }

        let mut bytes = [0; 12];
        DibitBytes::new(dibits.iter().cloned()).collect_slice_checked(&mut bytes[..]);

        Some(Ok(TSBK::new(bytes)))
    }
}

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
    GroupDataAnnounce,
    GroupDataAnnounceExplicit,
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
    LocRegistrationResponse,
    UnitRegistrationResponse,
    UnitRegistrationCommand,
    AuthCommand,
    DeregistrationAck,
    RoamingAddrCommand,
    RoamingAddrUpdate,
    SystemServiceBroadcast,
    AltControlBroadcast,
    RFSSStatusBroadcast,
    NetworkStatusBroadcast,
    AdjacentSiteBroadcast,
    ChannelParamsUpdate,
    ProtectionParamBroadcast,
    ProtectionParamUpdate,
    Reserved,
}

impl TSBKOpcode {
    pub fn from_bits(bits: u8) -> Option<TSBKOpcode> {
        use self::TSBKOpcode::*;

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
            0b010010 => Some(GroupDataAnnounce),
            0b010011 => Some(GroupDataAnnounceExplicit),
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
            0b101011 => Some(LocRegistrationResponse),
            0b101100 => Some(UnitRegistrationResponse),
            0b101101 => Some(UnitRegistrationCommand),
            0b101110 => Some(AuthCommand),
            0b101111 => Some(DeregistrationAck),

            0b110000...0b110101 => Some(Reserved),
            0b110110 => Some(RoamingAddrCommand),
            0b110111 => Some(RoamingAddrUpdate),

            0b111000 => Some(SystemServiceBroadcast),
            0b111001 => Some(AltControlBroadcast),
            0b111010 => Some(RFSSStatusBroadcast),
            0b111011 => Some(NetworkStatusBroadcast),
            0b111100 => Some(AdjacentSiteBroadcast),
            0b111101 => Some(ChannelParamsUpdate),
            0b111110 => Some(ProtectionParamBroadcast),
            0b111111 => Some(ProtectionParamUpdate),

            _ => None,
        }
    }
}

pub type Buf = [u8; 12];

#[derive(Copy, Clone)]
pub struct TSBK(Buf);

impl TSBK {
    pub fn new(buf: Buf) -> TSBK { TSBK(buf) }

    pub fn is_tail(&self) -> bool { self.0[0] >> 7 == 1 }
    pub fn protected(&self) -> bool { self.0[0] >> 6 & 1 == 1 }
    pub fn opcode(&self) -> Option<TSBKOpcode> { TSBKOpcode::from_bits(self.0[0] & 0x3F) }
    pub fn mfg(&self) -> u8 { self.0[1] }
    pub fn crc(&self) -> u16 { slice_u16(&self.0[10..]) }

    pub fn calc_crc(&self) -> u16 {
        crc::CRC16::new()
            .feed_bytes((&self.0[..10]).iter().cloned())
            .finish() as u16
    }
}

pub struct GroupVoiceGrant(Buf);

impl GroupVoiceGrant {
    pub fn new(tsbk: TSBK) -> Self { GroupVoiceGrant(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[5..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct GroupVoiceUpdate(Buf);

impl GroupVoiceUpdate {
    pub fn new(tsbk: TSBK) -> Self { GroupVoiceUpdate(tsbk.0) }

    pub fn channel_a(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn talk_group_a(&self) -> TalkGroup { TalkGroup::new(&self.0[4..]) }
    pub fn channel_b(&self) -> Channel { Channel::new(&self.0[6..]) }
    pub fn talk_group_b(&self) -> TalkGroup { TalkGroup::new(&self.0[8..]) }
}

pub struct GroupVoiceUpdateExplicit(Buf);

impl GroupVoiceUpdateExplicit {
    pub fn new(tsbk: TSBK) -> Self { GroupVoiceUpdateExplicit(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn tx_channel(&self) -> Channel { Channel::new(&self.0[4..]) }
    pub fn rx_channel(&self) -> Channel { Channel::new(&self.0[6..]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[8..]) }
}

pub struct UnitVoiceGrant(Buf);

impl UnitVoiceGrant {
    pub fn new(tsbk: TSBK) -> Self { UnitVoiceGrant(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitCallRequest(Buf);

impl UnitCallRequest {
    pub fn new(tsbk: TSBK) -> Self { UnitCallRequest(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_id(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitVoiceUpdate(Buf);

impl UnitVoiceUpdate {
    pub fn new(tsbk: TSBK) -> Self { UnitVoiceUpdate(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct PhoneGrant(Buf);

impl PhoneGrant {
    pub fn new(tsbk: TSBK) -> Self { PhoneGrant(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[3..]) }
    pub fn call_timer(&self) -> u16 { slice_u16(&self.0[5..]) }
    pub fn unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct UnitDataGrant(Buf);

impl UnitDataGrant {
    pub fn new(tsbk: TSBK) -> Self { UnitDataGrant(tsbk.0) }

    pub fn channel(&self) -> Channel { Channel::new(&self.0[2..]) }
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[4..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}
pub struct AltControlBroadcast(Buf);

impl AltControlBroadcast {
    pub fn new(tsbk: TSBK) -> Self { AltControlBroadcast(tsbk.0) }

    pub fn rfss(&self) -> u8 { self.0[2] }
    pub fn site(&self) -> u8 { self.0[3] }

    pub fn channel_a(&self) -> Channel { Channel::new(&self.0[4..]) }
    pub fn services_a(&self) -> SystemServices { SystemServices::new(self.0[6]) }

    pub fn channel_b(&self) -> Channel { Channel::new(&self.0[7..]) }
    pub fn serviced_b(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

pub struct NetworkStatusBroadcast(Buf);

impl NetworkStatusBroadcast {
    pub fn new(tsbk: TSBK) -> Self { NetworkStatusBroadcast(tsbk.0) }

    pub fn area(&self) -> u8 { self.0[2] }
    pub fn wacn(&self) -> u32 { slice_u24(&self.0[3..]) >> 4 }
    pub fn system(&self) -> u16 { slice_u16(&self.0[5..]) & 0xFFF }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7..]) }
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

pub struct SiteStatusBroadcast(Buf);

impl SiteStatusBroadcast {
    pub fn new(tsbk: TSBK) -> Self { SiteStatusBroadcast(tsbk.0) }

    pub fn area(&self) -> u8 { self.0[2] }
    pub fn is_conventional(&self) -> bool { self.0[3] & 0x80 != 0 }
    pub fn is_down(&self) -> bool { self.0[3] & 0x40 != 0 }
    pub fn is_current(&self) -> bool { self.0[3] & 0x20 != 0 }
    pub fn has_network(&self) -> bool { self.0[3] & 0x10 != 0 }
    pub fn system(&self) -> u16 { slice_u16(&self.0[3..]) & 0xFFF }
    pub fn rfss(&self) -> u8 { self.0[5] }
    pub fn site(&self) -> u8 { self.0[6] }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7..]) }
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

pub struct ChannelParamsUpdate(Buf);

impl ChannelParamsUpdate {
    pub fn new(tsbk: TSBK) -> Self { ChannelParamsUpdate(tsbk.0) }

    pub fn params(&self) -> ChannelParams {
        ChannelParams::new(self.base_freq(), self.channel(), self.bandwidth(),
                           self.offset(), self.spacing())
    }

    fn channel(&self) -> u8 { self.0[2] >> 4 }
    fn bandwidth(&self) -> u16 {
        (self.0[2] as u16 & 0xF) << 5 | (self.0[3] >> 3) as u16
    }
    fn offset(&self) -> u16 {
        (self.0[3] as u16 & 0x7) << 6 | (self.0[4] >> 2) as u16
    }
    fn spacing(&self) -> u16 {
        (self.0[4] as u16 & 0x3) << 8 | self.0[5] as u16
    }
    fn base_freq(&self) -> u32 { slice_u32(&self.0[6..]) }
}
