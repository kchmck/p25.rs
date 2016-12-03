/// Type of a data packet.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum DataPacketOpcode {
    ConfirmedPacket,
    UnconfirmedPacket,
    ResponsePacket,
    TrunkingPacket,
}

impl DataPacketOpcode {
    /// Convert a symbolic type to its associated identifier.
    pub fn to_bits(&self) -> u8 {
        use self::DataPacketOpcode::*;

        match *self {
            ConfirmedPacket => 0b10110,
            UnconfirmedPacket => 0b10101,
            ResponsePacket => 0b00011,
            TrunkingPacket => 0b10111,
        }
    }

    /// Parse a packet type from the given 5 bits.
    pub fn from_bits(bits: u8) -> Option<DataPacketOpcode> {
        use self::DataPacketOpcode::*;

        assert!(bits >> 5 == 0);

        match bits {
            0b10110 => Some(ConfirmedPacket),
            0b10101 => Some(UnconfirmedPacket),
            0b00011 => Some(ResponsePacket),
            0b10111 => Some(TrunkingPacket),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ServiceAccessPoint {
    UnencryptedUserData,
    EncryptedUserData,
    CircuitData,
    CircuitDataControl,
    PacketData,
    ARP,
    SNDCPControl,
    ExtendedAddressing,
    RegistrationAuth,
    ChannelReassignment,
    SystemConfiguration,
    Loopback,
    Statistics,
    OutOfService,
    Paging,
    Configuration,
    UnencryptedKeyManagement,
    EncryptedKeyManagement,
    TrunkingControl,
    EncryptedTrunkingControl,
}

impl ServiceAccessPoint {
    pub fn from_bits(bits: u8) -> Option<ServiceAccessPoint> {
        use self::ServiceAccessPoint::*;

        assert!(bits >> 6 == 0);

        match bits {
            0x00 => Some(UnencryptedUserData),
            0x01 => Some(EncryptedUserData),
            0x02 => Some(CircuitData),
            0x03 => Some(CircuitDataControl),
            0x04 => Some(PacketData),
            0x05 => Some(ARP),
            0x06 => Some(SNDCPControl),
            0x1F => Some(ExtendedAddressing),
            0x20 => Some(RegistrationAuth),
            0x21 => Some(ChannelReassignment),
            0x22 => Some(SystemConfiguration),
            0x23 => Some(Loopback),
            0x24 => Some(Statistics),
            0x25 => Some(OutOfService),
            0x26 => Some(Paging),
            0x27 => Some(Configuration),
            0x28 => Some(UnencryptedKeyManagement),
            0x29 => Some(EncryptedKeyManagement),
            0x3D => Some(TrunkingControl),
            0x3F => Some(EncryptedTrunkingControl),
            _ => None,
        }
    }

    pub fn to_bits(&self) -> u8 {
        use self::ServiceAccessPoint::*;

        match *self {
            UnencryptedUserData => 0x00,
            EncryptedUserData => 0x01,
            CircuitData => 0x02,
            CircuitDataControl => 0x03,
            PacketData => 0x04,
            ARP => 0x05,
            SNDCPControl => 0x06,
            ExtendedAddressing => 0x1F,
            RegistrationAuth => 0x20,
            ChannelReassignment => 0x21,
            SystemConfiguration => 0x22,
            Loopback => 0x23,
            Statistics => 0x24,
            OutOfService => 0x25,
            Paging => 0x26,
            Configuration => 0x27,
            UnencryptedKeyManagement => 0x28,
            EncryptedKeyManagement => 0x29,
            TrunkingControl => 0x3D,
            EncryptedTrunkingControl => 0x3F,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn test_sap_validate() {
        ServiceAccessPoint::from_bits(0b11111111);
    }
}
