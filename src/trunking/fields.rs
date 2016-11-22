use util::slice_u16;

pub struct ServiceOptions(u8);

impl ServiceOptions {
    pub fn new(opts: u8) -> ServiceOptions { ServiceOptions(opts) }

    pub fn emergency(&self) -> bool { self.0 >> 7 == 1 }
    pub fn protected(&self) -> bool { self.0 >> 6 & 1 == 1 }
    pub fn duplex(&self) -> bool { self.0 >> 5 & 1 == 1 }
    pub fn packet_switched(&self) -> bool { self.0 >> 4 & 1 == 1 }
    pub fn prio(&self) -> u8 { self.0 & 0x7 }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Channel(u16);

impl Channel {
    pub fn new(bytes: &[u8]) -> Channel { Channel(slice_u16(bytes)) }

    pub fn id(&self) -> u8 { (self.0 >> 12) as u8 }
    pub fn number(&self) -> u16 { self.0 & 0xFFF }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TalkGroup {
    Nobody,
    Default,
    Everbody,
    Other(u16),
}

impl TalkGroup {
    pub fn new(bytes: &[u8]) -> TalkGroup {
        Self::from_bits(slice_u16(bytes))
    }

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
    /// Whether site is in failure state.
    pub fn failing(&self) -> bool { self.0 & 0b100 != 0 }
    /// Whether this information is up-to-date (broadcasting site is in communication with
    /// adjacent site.)
    pub fn current(&self) -> bool { self.0 & 0b10 != 0 }
    /// Whether site has active network connection with RFSS controller and can
    /// communicate with other sites.
    pub fn networked(&self) -> bool { self.0 & 1 != 0 }
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
}
