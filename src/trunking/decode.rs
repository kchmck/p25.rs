pub struct ServiceOptions(u8);

impl ServiceOptions {
    pub fn new(opts: u8) -> ServiceOptions { ServiceOptions(opts) }

    pub fn emergency(&self) -> bool { self.0 >> 7 == 1 }
    pub fn protected(&self) -> bool { self.0 >> 6 & 1 == 1 }
    pub fn duplex(&self) -> bool { self.0 >> 5 & 1 == 1 }
    pub fn packet_switched(&self) -> bool { self.0 >> 4 & 1 == 1 }
    pub fn prio(&self) -> u8 { self.0 & 0x7 }
}

pub struct Channel(u16);

impl Channel {
    pub fn new(bytes: &[u8]) -> Channel { Channel(slice_u16(bytes)) }

    pub fn band(&self) -> u8 { (self.0 >> 12) as u8 }
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ChannelParams {
    /// Receive frequency in Hz.
    pub rx_freq: u64,
    /// Transmit frequency in Hz.
    pub tx_freq: u64,
    /// Channel bandwidth in Hz.
    pub bandwidth: u64,
}

impl ChannelParams {
    pub fn new(base: u32, channel: u8, bandwidth: u16, offset: u16, spacing: u16)
        -> ChannelParams
    {
        let rx = base as u64 * 5 + channel as u64 * spacing as u64 * 125;
        let off = (offset as u64 & 0xFF) * 250_000;
        let tx = if offset >> 8 == 0 {
            rx - off
        } else {
            rx + off
        };

        ChannelParams {
            rx_freq: rx,
            tx_freq: tx,
            bandwidth: bandwidth as u64 * 125,
        }
    }
}

pub fn slice_u16(bytes: &[u8]) -> u16 {
    (bytes[0] as u16) << 8 | bytes[1] as u16
}

pub fn slice_u24(bytes: &[u8]) -> u32 {
    (slice_u16(bytes) as u32) << 8 | bytes[2] as u32
}

pub fn slice_u32(bytes: &[u8]) -> u32 {
    (slice_u16(bytes) as u32) << 16 | slice_u16(&bytes[2..]) as u32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slice_u16() {
        assert_eq!(slice_u16(&[0xDE, 0xAD]), 0xDEAD);
        assert_eq!(slice_u16(&[0xAB, 0xCD, 0xEF]), 0xABCD);
    }

    #[test]
    fn test_slice_u24() {
        assert_eq!(slice_u24(&[0xDE, 0xAD, 0xBE]), 0xDEADBE);
        assert_eq!(slice_u24(&[0xAB, 0xCD, 0xEF, 0x12]), 0xABCDEF);
    }

    #[test]
    fn test_slice_u32() {
        assert_eq!(slice_u32(&[0xDE, 0xAD, 0xBE, 0xEF]), 0xDEADBEEF);
        assert_eq!(slice_u32(&[0xDE, 0xAD, 0xBE, 0xEF, 0x12]), 0xDEADBEEF);
    }

    #[test]
    fn test_channel_params() {
        let p = ChannelParams::new(170201250, 0, 0x64, 0b010110100, 0x32);
        assert_eq!(p.rx_freq, 851_006_250);
        assert_eq!(p.tx_freq, 806_006_250);
        assert_eq!(p.bandwidth, 12500);
    }
}
