use bits::Dibit;
use buffer;
use coding::bch;
use error::{Result, P25Error};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NetworkAccessCode {
    Default,
    ReceiveAny,
    RepeatAny,
    Other(u16),
}

impl NetworkAccessCode {
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

    pub fn to_bits(&self) -> u16 {
        use self::NetworkAccessCode::*;

        match *self {
            Default => 0x293,
            ReceiveAny => 0xF7E,
            RepeatAny => 0xF7F,
            Other(bits) => bits,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataUnit {
    VoiceHeader,
    VoiceSimpleTerminator,
    VoiceLCTerminator,
    VoiceLCFrameGroup,
    VoiceCCFrameGroup,
    DataPacket,
    TrunkingSignaling,
}

impl DataUnit {
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

    pub fn to_bits(&self) -> u8 {
        use self::DataUnit::*;

        match *self {
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

#[derive(Copy, Clone, Debug)]
pub struct NetworkID {
    access_code: NetworkAccessCode,
    data_unit: DataUnit,
}

impl NetworkID {
    pub fn new(access_code: u16, data_unit: DataUnit) -> NetworkID {
        NetworkID {
            access_code: NetworkAccessCode::from_bits(access_code),
            data_unit: data_unit,
        }
    }

    pub fn from_bits(bits: u16) -> Option<NetworkID> {
        match DataUnit::from_bits(bits as u8 & 0b1111) {
            Some(du) => Some(NetworkID::new(bits >> 4, du)),
            None => None,
        }
    }

    pub fn to_bits(&self) -> u16 {
        (self.access_code.to_bits() as u16) << 4 | self.data_unit.to_bits() as u16
    }

    pub fn access_code(&self) -> NetworkAccessCode { self.access_code }
    pub fn data_unit(&self) -> DataUnit { self.data_unit }

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

pub struct NIDReceiver {
    dibits: buffer::Buffer<buffer::DibitStorage>,
}

impl NIDReceiver {
    pub fn new() -> NIDReceiver {
        NIDReceiver {
            dibits: buffer::Buffer::new(buffer::DibitStorage::new(32)),
        }
    }

    pub fn feed(&mut self, dibit: Dibit) -> Option<Result<NetworkID>> {
        let buf = match self.dibits.feed(dibit) {
            Some(buf) => *buf,
            None => return None,
        };

        self.dibits.reset();

        let data = match bch::decode(buf) {
            Some((data, err)) => data,
            None => return Some(Err(P25Error::BCHUnrecoverable)),
        };

        match NetworkID::from_bits(data) {
            Some(nid) => Some(Ok(nid)),
            None => Some(Err(P25Error::UnknownNID)),
        }
    }
}
