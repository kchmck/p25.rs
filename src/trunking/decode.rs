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
}
