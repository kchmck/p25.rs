use collect_slice::CollectSlice;

use bits::{Dibit, DibitBytes};
use buffer::{Buffer, DataPayloadStorage};
use coding::trellis;
use consts::{TSBK_DIBITS, TSBK_BYTES};
use data::{crc, interleave};
use error::{Result, P25Error};
use util::{slice_u16, slice_u24, slice_u32};

use trunking::decode::*;

pub struct TSBKReceiver {
    dibits: Buffer<DataPayloadStorage>,
}

impl TSBKReceiver {
    pub fn new() -> TSBKReceiver {
        TSBKReceiver {
            dibits: Buffer::new(DataPayloadStorage::new()),
        }
    }

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

pub type Buf = [u8; TSBK_BYTES];

#[derive(Copy, Clone)]
pub struct TSBKFields(Buf);

impl TSBKFields {
    pub fn new(buf: Buf) -> TSBKFields { TSBKFields(buf) }

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

    pub fn crc_valid(&self) -> bool {
        self.crc() == self.calc_crc()
    }
}

pub struct GroupVoiceGrant(Buf);

impl GroupVoiceGrant {
    pub fn new(tsbk: TSBKFields) -> Self { GroupVoiceGrant(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[5..]) }
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[7..]) }
}

pub struct GroupVoiceUpdate(Buf);

impl GroupVoiceUpdate {
    pub fn new(tsbk: TSBKFields) -> Self { GroupVoiceUpdate(tsbk.0) }

    pub fn updates(&self) -> [(TalkGroup, Channel); 2] {
        [
            (self.talk_group_a(), self.channel_a()),
            (self.talk_group_b(), self.channel_b()),
        ]
    }

    fn channel_a(&self) -> Channel { Channel::new(&self.0[2..]) }
    fn talk_group_a(&self) -> TalkGroup { TalkGroup::new(&self.0[4..]) }
    fn channel_b(&self) -> Channel { Channel::new(&self.0[6..]) }
    fn talk_group_b(&self) -> TalkGroup { TalkGroup::new(&self.0[8..]) }
}

pub struct GroupVoiceUpdateExplicit(Buf);

impl GroupVoiceUpdateExplicit {
    pub fn new(tsbk: TSBKFields) -> Self { GroupVoiceUpdateExplicit(tsbk.0) }

    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[2]) }
    pub fn tx_channel(&self) -> Channel { Channel::new(&self.0[4..]) }
    pub fn rx_channel(&self) -> Channel { Channel::new(&self.0[6..]) }
    pub fn talk_group(&self) -> TalkGroup { TalkGroup::new(&self.0[8..]) }
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

pub struct AltControlChannel(Buf);

impl AltControlChannel {
    pub fn new(tsbk: TSBKFields) -> Self { AltControlChannel(tsbk.0) }

    pub fn rfss(&self) -> u8 { self.0[2] }
    pub fn site(&self) -> u8 { self.0[3] }

    pub fn channels(&self) -> [(Channel, SystemServices); 2] {
        [
            (self.channel_a(), self.services_a()),
            (self.channel_b(), self.serviced_b()),
        ]
    }

    fn channel_a(&self) -> Channel { Channel::new(&self.0[4..]) }
    fn services_a(&self) -> SystemServices { SystemServices::new(self.0[6]) }

    fn channel_b(&self) -> Channel { Channel::new(&self.0[7..]) }
    fn serviced_b(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

/// Carries Site and RFSS information of current control channel.
pub struct RFSSStatusBroadcast(Buf);

impl RFSSStatusBroadcast {
    /// Create a new `RFSSStatusBroadcast` decoder from base TSBK decoder.
    pub fn new(tsbk: TSBKFields) -> Self { RFSSStatusBroadcast(tsbk.0) }

    /// Location registration area of current site.
    pub fn area(&self) -> u8 { self.0[2] }
    /// Whether the site is networked with the RFSS controller, which determines if it can
    /// communicate with other sites.
    pub fn networked(&self) -> bool { self.0[3] & 0b10000 != 0 }
    /// System ID of current site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[3...4]) & 0xFFF }
    /// RF Subsystem ID of current site within System.
    pub fn rfss(&self) -> u8 { self.0[5] }
    /// Site ID of current site within RFSS.
    pub fn site(&self) -> u8 { self.0[6] }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7...8]) }
    /// Services supported by the current site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

pub struct NetworkStatusBroadcast(Buf);

impl NetworkStatusBroadcast {
    pub fn new(tsbk: TSBKFields) -> Self { NetworkStatusBroadcast(tsbk.0) }

    pub fn area(&self) -> u8 { self.0[2] }
    pub fn wacn(&self) -> u32 { slice_u24(&self.0[3..]) >> 4 }
    pub fn system(&self) -> u16 { slice_u16(&self.0[5..]) & 0xFFF }
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7..]) }
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

/// Status of current site.
pub struct SiteStatusBroadcast(Buf);

impl SiteStatusBroadcast {
    /// Create a new `SiteStatusBroadcast` decoder from base TSBK decoder.
    pub fn new(tsbk: TSBKFields) -> Self { SiteStatusBroadcast(tsbk.0) }

    /// Location registration area of site.
    pub fn area(&self) -> u8 { self.0[2] }
    /// Properties of current site.
    pub fn opts(&self) -> SiteOptions { SiteOptions::new(self.0[3] >> 4) }
    /// System ID of site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[3..]) & 0xFFF }
    /// RF Subsystem ID of site within System.
    pub fn rfss(&self) -> u8 { self.0[5] }
    /// Site ID of site within RFSS.
    pub fn site(&self) -> u8 { self.0[6] }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7..]) }
    /// Services supported by the site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

/// Advertisement of an adjacent/nearby site within the same WACN (Wide Area Communication
/// Network.)
pub struct AdjacentSite(Buf);

impl AdjacentSite {
    /// Create a new `AdjacentSite` decoder from base TSBK decoder.
    pub fn new(tsbk: TSBKFields) -> Self { AdjacentSite(tsbk.0) }

    /// Location registration area of adjacent site, which determines whether a subscriber
    /// must update the network before roaming to the site.
    pub fn area(&self) -> u8 { self.0[2] }
    /// Description of adjacent site.
    pub fn opts(&self) -> SiteOptions { SiteOptions::new(self.0[3] >> 4) }
    /// System ID of adjacent site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[3...4]) & 0xFFF }
    /// RF Subsystem ID of adjacent site within the System.
    pub fn rfss(&self) -> u8 { self.0[5] }
    /// Site ID of adjacent site within the RFSS.
    pub fn site(&self) -> u8 { self.0[6] }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[7...8]) }
    /// Services supported by the adjacent site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[9]) }
}

pub struct ChannelParamsUpdate(Buf);

impl ChannelParamsUpdate {
    pub fn new(tsbk: TSBKFields) -> Self { ChannelParamsUpdate(tsbk.0) }

    pub fn id(&self) -> u8 { self.0[2] >> 4 }

    pub fn params(&self) -> ChannelParams {
        ChannelParams::new(self.base(), self.bandwidth(), self.offset(), self.spacing())
    }

    fn bandwidth(&self) -> u16 {
        (self.0[2] as u16 & 0xF) << 5 | (self.0[3] >> 3) as u16
    }

    fn offset(&self) -> u16 {
        (self.0[3] as u16 & 0x7) << 6 | (self.0[4] >> 2) as u16
    }

    fn spacing(&self) -> u16 {
        (self.0[4] as u16 & 0x3) << 8 | self.0[5] as u16
    }

    fn base(&self) -> u32 { slice_u32(&self.0[6..]) }
}
