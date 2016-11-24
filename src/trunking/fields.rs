//! Decode various trunking-related packet fields.

use util::{slice_u16, slice_u24, slice_u32};

/// Options that can be requested/granted by a service.
pub struct ServiceOptions(u8);

impl ServiceOptions {
    /// Create a new `ServiceOptions` based on the given byte.
    pub fn new(opts: u8) -> ServiceOptions { ServiceOptions(opts) }

    /// Whether the service should be processed as an emergency.
    pub fn emergency(&self) -> bool { self.0 >> 7 == 1 }
    /// Whether the channel should be encrypted.
    pub fn protected(&self) -> bool { self.0 >> 6 & 1 == 1 }
    /// Whether the channel should be full duplex for simultaneous transmit and receive
    /// (otherwise fall back to half duplex.)
    pub fn full_duplex(&self) -> bool { self.0 >> 5 & 1 == 1 }
    /// Whether the service should be packet switched (otherwise fall back to circuit
    /// switched.)
    pub fn packet_switched(&self) -> bool { self.0 >> 4 & 1 == 1 }
    /// Priority assigned to service, with 1 as lowest and 7 as highest.
    pub fn prio(&self) -> u8 { self.0 & 0x7 }
}

/// Uniquely identifies a channel within a site.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Channel(u16);

impl Channel {
    /// Create a new `Channel` from the given 16 bits.
    pub fn new(bytes: &[u8]) -> Channel { Channel(slice_u16(bytes)) }

    /// Channel ID whose parameters to use.
    pub fn id(&self) -> u8 { (self.0 >> 12) as u8 }
    /// Individual channel number within the channel.
    pub fn number(&self) -> u16 { self.0 & 0xFFF }
}

/// Identifies which group a message belongs to.
///
/// In a production P25 system, users can set their radios to receive one or more
/// talkgroups, and the radio will only unsquelch if one of those talkgroups is seen.
/// Additionally, the user directs each transmission to a talkgroup selected on the
/// radio.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TalkGroup {
    /// Includes nobody.
    Nobody,
    /// Default talkgroup when no other is selected.
    Default,
    /// Includes everybody.
    Everbody,
    /// Specific group of users.
    Other(u16),
}

impl TalkGroup {
    /// Parse a talkgroup from the given 16 bit slice.
    pub fn new(bytes: &[u8]) -> TalkGroup {
        Self::from_bits(slice_u16(bytes))
    }

    /// Parse a talkgroup from the given 16 bits.
    pub fn from_bits(bits: u16) -> TalkGroup {
        use self::TalkGroup::*;

        match bits {
            0x0000 => Nobody,
            0x0001 => Default,
            0xFFFF => Everbody,
            _ => Other(bits),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SystemServices(u8);

impl SystemServices {
    pub fn new(ssc: u8) -> Self { SystemServices(ssc) }

    pub fn is_composite(&self) -> bool { self.0 & 0x01 != 0 }
    pub fn updates_only(&self) -> bool { self.0 & 0x02 != 0 }
    pub fn is_backup(&self) -> bool { self.0 & 0x04 != 0 }
    pub fn has_data(&self) -> bool { self.0 & 0x10 != 0 }
    pub fn has_voice(&self) -> bool { self.0 & 0x20 != 0 }
    pub fn has_registration(&self) -> bool { self.0 & 0x40 != 0 }
    pub fn has_auth(&self) -> bool { self.0 & 0x80 != 0 }
}

/// Maps channel identifiers (maximum 16 per control channel) to their tuning parameters.
pub type ChannelParamsMap = [Option<ChannelParams>; 16];

/// Computes TX/RX frequencies and bandwidth for channel numbers within a site.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChannelParams {
    /// Base frequency in Hz.
    base: u32,
    /// Channel spacing in Hz.
    spacing: u32,
    /// Transmit frequency offset in Hz.
    offset: i32,
    /// Channel bandwidth in Hz.
    pub bandwidth: u32,
}

impl ChannelParams {
    /// Create a new `ChannelParams` from the given base frequency (5Hz steps), bandwidth
    /// (125Hz steps), TX offset (250kHz steps), and inter-channel spacing (125Hz steps.)
    pub fn new(base: u32, bandwidth: u16, offset: u16, spacing: u16) -> ChannelParams {
        // The MSB denotes the sign and the lower byte is the actual offset.
        let off = (offset as i32 & 0xFF) * 250_000;

        ChannelParams {
            base: base * 5,
            spacing: spacing as u32 * 125,
            offset: if offset >> 8 == 0 { -off } else { off },
            bandwidth: bandwidth as u32 * 125,
        }
    }

    /// Receive frequency for the given channel number in Hz.
    pub fn rx_freq(&self, ch: u16) -> u32 {
        self.base + self.spacing * ch as u32
    }

    /// Transmit frequency for the given channel number in Hz.
    pub fn tx_freq(&self, ch: u16) -> u32 {
        self.rx_freq(ch) + self.offset as u32
    }
}

/// Options for a P25 site.
pub struct SiteOptions(u8);

impl SiteOptions {
    /// Create a new `SiteOptions` from the given 4-bit word.
    pub fn new(opts: u8) -> SiteOptions {
        assert!(opts >> 4 == 0);
        SiteOptions(opts)
    }

    /// Whether site is "conventional", with no trunking.
    pub fn conventional(&self) -> bool { self.0 & 0b1000 != 0 }
    /// Whether site is in a failure state.
    pub fn failing(&self) -> bool { self.0 & 0b100 != 0 }
    /// Whether this information is up-to-date (whether broadcasting site is in
    /// communication with adjacent site.)
    pub fn current(&self) -> bool { self.0 & 0b10 != 0 }
    /// Whether site has active network connection with RFSS controller and can
    /// communicate with other sites.
    pub fn networked(&self) -> bool { self.0 & 1 != 0 }
}

/// Updates subscribers about new or ongoing talkgroup conversations.
///
/// Note that this can be used for both `GroupVoiceUpdate` and `GroupDataUpdate`.
pub struct GroupTrafficUpdate<'a>(&'a [u8]);

impl<'a> GroupTrafficUpdate<'a> {
    /// Create a new `GroupTrafficUpdate` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { GroupTrafficUpdate(payload) }

    /// Retrieve the set of active talkgroups included in the update along with the
    /// parameters for tuning to the traffic channel of each.
    pub fn updates(&self) -> [(Channel, TalkGroup); 2] {
        [
            (Channel::new(&self.0[0...1]), TalkGroup::new(&self.0[2...3])),
            (Channel::new(&self.0[4...5]), TalkGroup::new(&self.0[6...7])),
        ]
    }
}

/// Advertisement of an adjacent/nearby site within the same WACN (Wide Area Communication
/// Network.)
pub struct AdjacentSite<'a>(&'a [u8]);

impl<'a> AdjacentSite<'a> {
    /// Create a new `AdjacentSite` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { AdjacentSite(payload) }

    /// Location registration area of adjacent site, which determines whether a subscriber
    /// must update the network before roaming to the site.
    pub fn area(&self) -> u8 { self.0[0] }
    /// Description of adjacent site.
    pub fn opts(&self) -> SiteOptions { SiteOptions::new(self.0[1] >> 4) }
    /// System ID of adjacent site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[1...2]) & 0xFFF }
    /// RF Subsystem ID of adjacent site within the System.
    pub fn rfss(&self) -> u8 { self.0[3] }
    /// Site ID of adjacent site within the RFSS.
    pub fn site(&self) -> u8 { self.0[4] }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[5...6]) }
    /// Services supported by the adjacent site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[7]) }
}

/// Advertisement of parameters used to calculate TX/RX frequencies within the given
/// associated channel.
pub struct ChannelParamsUpdate<'a>(&'a [u8]);

impl<'a> ChannelParamsUpdate<'a> {
    /// Create a new `ChannelParamsUpdate` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { ChannelParamsUpdate(payload) }

    /// Channel ID associated with the enclosed parameters (can be up to 16 per control
    /// channel.)
    pub fn id(&self) -> u8 { self.0[0] >> 4 }

    /// Parameters for the associated channel.
    pub fn params(&self) -> ChannelParams {
        ChannelParams::new(self.base(), self.bandwidth(), self.offset(), self.spacing())
    }

    /// Bandwidth in steps of 125Hz.
    fn bandwidth(&self) -> u16 {
        (self.0[0] as u16 & 0xF) << 5 | (self.0[1] >> 3) as u16
    }

    /// Offset of TX frequency from base RX frequency in steps of 250kHz.
    fn offset(&self) -> u16 {
        (self.0[1] as u16 & 0x7) << 6 | (self.0[2] >> 2) as u16
    }

    /// Spacing between individual channel numbers in steps of 125Hz.
    fn spacing(&self) -> u16 {
        (self.0[2] as u16 & 0x3) << 8 | self.0[3] as u16
    }

    /// Base RX frequency in steps of 5Hz.
    fn base(&self) -> u32 { slice_u32(&self.0[4...7]) }
}

/// Advertisement of one or more alternative control channels for the current site.
pub struct AltControlChannel<'a>(&'a [u8]);

impl<'a> AltControlChannel<'a> {
    /// Create a new `AltControlChannel` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { AltControlChannel(payload) }

    /// RF Subsystem ID of current site within System.
    pub fn rfss(&self) -> u8 { self.0[0] }
    /// Site ID of current site within RFSS.
    pub fn site(&self) -> u8 { self.0[1] }

    /// Retrieve alternative sites, with each site's tuning parameters and supported
    /// services.
    pub fn alts(&self) -> [(Channel, SystemServices); 2] {
        [
            (Channel::new(&self.0[2...3]), SystemServices::new(self.0[4])),
            (Channel::new(&self.0[5...6]), SystemServices::new(self.0[7])),
        ]
    }
}

/// Site and RFSS information of current control channel.
pub struct RFSSStatusBroadcast<'a>(&'a [u8]);

impl<'a> RFSSStatusBroadcast<'a> {
    /// Create a new `RFSSStatusBroadcast` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { RFSSStatusBroadcast(payload) }

    /// Location registration area of current site.
    pub fn area(&self) -> u8 { self.0[0] }
    /// Whether the site is networked with the RFSS controller, which determines if it can
    /// communicate with other sites.
    pub fn networked(&self) -> bool { self.0[1] & 0b10000 != 0 }
    /// System ID of current site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[1...2]) & 0xFFF }
    /// RF Subsystem ID of current site within System.
    pub fn rfss(&self) -> u8 { self.0[3] }
    /// Site ID of current site within RFSS.
    pub fn site(&self) -> u8 { self.0[4] }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[5...6]) }
    /// Services supported by the current site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[7]) }
}

/// WACN (Wide Area Communication Network) and System ID information of current control
/// channel.
pub struct NetworkStatusBroadcast<'a>(&'a [u8]);

impl<'a> NetworkStatusBroadcast<'a> {
    /// Create a new `NetworkStatusBroadcast` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { NetworkStatusBroadcast(payload) }

    /// Location registration area of site.
    pub fn area(&self) -> u8 { self.0[0] }
    /// WACN ID within the communications network.
    pub fn wacn(&self) -> u32 { slice_u24(&self.0[1...3]) >> 4 }
    /// System ID of site within WACN.
    pub fn system(&self) -> u16 { slice_u16(&self.0[3...4]) & 0xFFF }
    /// Channel information for computing TX/RX frequencies.
    pub fn channel(&self) -> Channel { Channel::new(&self.0[5...6]) }
    /// Services supported by the current site.
    pub fn services(&self) -> SystemServices { SystemServices::new(self.0[7]) }
}

/// Registration response.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum RegResponse {
    /// Registration is accepted.
    Accept,
    /// RFSS was unable to verify registration.
    Fail,
    /// Registration isn't allowed at this location.
    Deny,
    /// Denied temporarily, but user may retry registration.
    Refuse,
}

impl RegResponse {
    /// Try to parse a registration response from the given 2 bits.
    pub fn from_bits(bits: u8) -> RegResponse {
        use self::RegResponse::*;

        assert!(bits >> 2 == 0);

        match bits {
            0b00 => Accept,
            0b01 => Fail,
            0b10 => Deny,
            0b11 => Refuse,
            _ => unreachable!(),
        }
    }
}

/// Request for a target unit to call a source unit.
pub struct UnitCallAlert<'a>(&'a [u8]);

impl<'a> UnitCallAlert<'a> {
    /// Create a new `UnitCallAlert` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { UnitCallAlert(payload) }

    /// Target unit.
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[2...4]) }
    /// Requesting unit.
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[5...7]) }
}

/// Signals a target unit that a unit-to-unit all has been requested.
pub struct UnitCallRequest<'a>(&'a [u8]);

impl<'a> UnitCallRequest<'a> {
    /// Create a new `UnitCallRequest` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { UnitCallRequest(payload) }

    /// Options requested/granted for resulting channel.
    pub fn opts(&self) -> ServiceOptions { ServiceOptions::new(self.0[0]) }
    /// Target unit.
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[2...4]) }
    /// Requesting unit.
    pub fn src_unit(&self) -> u32 { slice_u24(&self.0[5...7]) }
}

/// Alerts a unit of a call from the public phone network.
pub struct PhoneAlert<'a>(&'a [u8]);

impl<'a> PhoneAlert<'a> {
    /// Create a new `PhoneAlert` decoder from the given payload bytes.
    pub fn new(payload: &'a [u8]) -> Self { PhoneAlert(payload) }

    /// The 10-digit phone number of the calling party, as encoded bytes.
    pub fn digits(&self) -> &[u8] { &self.0[0...4] }
    /// Unit the call is for.
    pub fn dest_unit(&self) -> u32 { slice_u24(&self.0[5...7]) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_channel_params() {
        // Example from the standard.
        let p = ChannelParams::new(170201250, 0x64, 0b010110100, 0x32);
        assert_eq!(p.base, 851_006_250);
        assert_eq!(p.spacing, 6_250);
        assert_eq!(p.offset, -45_000_000);
        assert_eq!(p.bandwidth, 12_500);
        assert_eq!(p.rx_freq(0b1001), 851_062_500);
    }

    #[test]
    fn test_group_traffic_updates() {
        let buf = [
            0b10001000,
            0b01110111,
            0b11111111,
            0b11111111,
            0b10010001,
            0b00000001,
            0b10101010,
            0b10101010,
        ];

        let u = GroupTrafficUpdate(&buf[..]).updates();

        assert_eq!(u[0].0.id(), 0b1000);
        assert_eq!(u[0].0.number(), 0b100001110111);
        assert_eq!(u[0].1, TalkGroup::Everbody);
        assert_eq!(u[1].0.id(), 0b1001);
        assert_eq!(u[1].0.number(), 0b000100000001);
        assert_eq!(u[1].1, TalkGroup::Other(0b1010101010101010));
    }
}
