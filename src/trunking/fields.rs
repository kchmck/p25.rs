use util::{slice_u16, slice_u32};

pub struct ServiceOptions(u8);

impl ServiceOptions {
    pub fn new(opts: u8) -> ServiceOptions { ServiceOptions(opts) }

    pub fn emergency(&self) -> bool { self.0 >> 7 == 1 }
    pub fn protected(&self) -> bool { self.0 >> 6 & 1 == 1 }
    pub fn duplex(&self) -> bool { self.0 >> 5 & 1 == 1 }
    pub fn packet_switched(&self) -> bool { self.0 >> 4 & 1 == 1 }
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
    pub fn has_updates(&self) -> bool { self.0 & 0x02 != 0 }
    pub fn is_backup(&self) -> bool { self.0 & 0x04 != 0 }
    pub fn has_data(&self) -> bool { self.0 & 0x10 != 0 }
    pub fn has_voice(&self) -> bool { self.0 & 0x20 != 0 }
    pub fn has_registration(&self) -> bool { self.0 & 0x40 != 0 }
    pub fn has_auth(&self) -> bool { self.0 & 0x80 != 0 }
}

/// Map channel identifier (maximum 16 per control channel) to its parameters.
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

/// Represents the channel updates seen in a Group Voice Update.
pub type ChannelUpdates = [(Channel, TalkGroup); 2];

/// Parse out the pair of channels/talkgroups found in a Group Voice Update.
pub fn parse_updates(buf: &[u8]) -> ChannelUpdates {
    [
        (Channel::new(&buf[0...1]), TalkGroup::new(&buf[2...3])),
        (Channel::new(&buf[4...5]), TalkGroup::new(&buf[6...7])),
    ]
}

/// Advertisement of an adjacent/nearby site within the same WACN (Wide Area Communication
/// Network.)
pub struct AdjacentSite<'a>(&'a [u8]);

impl<'a> AdjacentSite<'a> {
    /// Create a new `AdjacentSite` decoder from given payload bytes decoder.
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
    /// Create a new `ChannelParamsUpdate` decoder from given payload bytes.
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
    fn test_parse_updates() {
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

        let u = parse_updates(&buf[..]);

        assert_eq!(u[0].0.id(), 0b1000);
        assert_eq!(u[0].0.number(), 0b100001110111);
        assert_eq!(u[0].1, TalkGroup::Everbody);
        assert_eq!(u[1].0.id(), 0b1001);
        assert_eq!(u[1].0.number(), 0b000100000001);
        assert_eq!(u[1].1, TalkGroup::Other(0b1010101010101010));
    }
}
