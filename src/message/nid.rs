//! Network ID (NID), Network Access Code (NAC), and Data Unit utilities.

use bits::Dibit;
use buffer;
use coding::bch;
use error::{Result, P25Error};

/// "Digital squelch" NAC field of the NID.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NetworkAccessCode {
    /// Default P25 NAC.
    Default,
    /// Allows receiver to unsquelch on any NAC (shouldn't be transmitted.)
    ReceiveAny,
    /// Allows repeater to unsquelch/retransmit any NAC (shouldn't be transmitted.)
    RepeatAny,
    /// Custom NAC.
    Other(u16),
}

impl NetworkAccessCode {
    /// Parse 12 bits into a NAC.
    pub fn from_bits(bits: u16) -> NetworkAccessCode {
        use self::NetworkAccessCode::*;

        assert!(bits >> 12 == 0);

        match bits {
            0x293 => Default,
            0xF7E => ReceiveAny,
            0xF7F => RepeatAny,
            _ => Other(bits),
        }
    }

    /// Convert NAC to a 12-bit word.
    pub fn to_bits(self) -> u16 {
        use self::NetworkAccessCode::*;

        match self {
            Default => 0x293,
            ReceiveAny => 0xF7E,
            RepeatAny => 0xF7F,
            Other(bits) => bits,
        }
    }
}

/// Data unit of associated packet.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataUnit {
    /// Voice header packet.
    VoiceHeader,
    /// Simple terminator packet.
    VoiceSimpleTerminator,
    /// Terminator packet with link control word.
    VoiceLCTerminator,
    /// Link control voice frame group.
    VoiceLCFrameGroup,
    /// Crypto control voice frame group.
    VoiceCCFrameGroup,
    /// Confirmed/Unconfirmed data packet
    DataPacket,
    /// Trunking signalling packet.
    TrunkingSignaling,
}

impl DataUnit {
    /// Parse 4 bits into a data unit type.
    pub fn from_bits(bits: u8) -> Option<DataUnit> {
        use self::DataUnit::*;

        assert!(bits >> 4 == 0);

        match bits {
            0b0000 => Some(VoiceHeader),
            0b0011 => Some(VoiceSimpleTerminator),
            0b1111 => Some(VoiceLCTerminator),
            0b0101 => Some(VoiceLCFrameGroup),
            0b1010 => Some(VoiceCCFrameGroup),
            0b1100 => Some(DataPacket),
            0b0111 => Some(TrunkingSignaling),
            _ => None,
        }
    }

    /// Convert data unit to 4-bit word.
    pub fn to_bits(self) -> u8 {
        use self::DataUnit::*;

        match self {
            VoiceHeader => 0b0000,
            VoiceSimpleTerminator => 0b0011,
            VoiceLCTerminator => 0b1111,
            VoiceLCFrameGroup => 0b0101,
            VoiceCCFrameGroup => 0b1010,
            DataPacket => 0b1100,
            TrunkingSignaling => 0b0111,
        }
    }
}

/// NID word associated with each P25 packet.
#[derive(Copy, Clone, Debug)]
pub struct NetworkID {
    /// NAC field.
    pub access_code: NetworkAccessCode,
    /// DUID field.
    pub data_unit: DataUnit,
}

impl NetworkID {
    /// Create an NID word from the given NAC and data unit.
    pub fn new(access_code: NetworkAccessCode, data_unit: DataUnit) -> NetworkID {
        NetworkID {
            access_code: access_code,
            data_unit: data_unit,
        }
    }

    /// Parse NID from the given 16-bit word.
    pub fn from_bits(bits: u16) -> Option<NetworkID> {
        match DataUnit::from_bits(bits as u8 & 0b1111) {
            Some(du) => Some(NetworkID::new(NetworkAccessCode::from_bits(bits >> 4), du)),
            None => None,
        }
    }

    /// Convert NID to 16-bit representation.
    pub fn to_bits(&self) -> u16 {
        (self.access_code.to_bits() as u16) << 4 | self.data_unit.to_bits() as u16
    }

    /// Encode NID into a byte sequence.
    pub fn encode(&self) -> [u8; 8] {
        let bits = self.to_bits();
        let e = bch::encode(bits);

        [
            (e >> 56) as u8,
            (e >> 48) as u8,
            (e >> 40) as u8,
            (e >> 32) as u8,
            (e >> 24) as u8,
            (e >> 16) as u8,
            (e >> 8) as u8,
            e as u8,
        ]
    }
}

/// State machine that attempts to parse a stream of dibits into an NID word.
pub struct NIDReceiver {
    /// Buffered dibits.
    dibits: buffer::Buffer<buffer::NIDStorage>,
}

impl NIDReceiver {
    /// Create a new `NIDReceiver` with an empty buffer.
    pub fn new() -> NIDReceiver {
        NIDReceiver {
            dibits: buffer::Buffer::new(buffer::NIDStorage::new()),
        }
    }

    /// Feed in a data symbol, possibly producing a decoded NID. Return `Some(Ok(nid))` if
    /// an NID was successfully parsed, `Some(Err(err))` if an unrecoverable error
    /// occurred, and `None` for no event.
    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<NetworkID>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf,
            None => return None,
        };

        let data = match bch::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(P25Error::BchUnrecoverable)),
        };

        match NetworkID::from_bits(data) {
            Some(nid) => Some(Ok(nid)),
            None => Some(Err(P25Error::UnknownNID)),
        }
    }
}
