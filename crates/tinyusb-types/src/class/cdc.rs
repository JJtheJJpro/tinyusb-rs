//! USB Communication Device Class (CDC) definitions.
use byteorder::{ByteOrder, LittleEndian};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum CdcSubclass {
    DirectLineControlModel = 0x01,
    AbstractControlModel = 0x02,
    TelephoneControlModel = 0x03,
    MultiChannelControlModel = 0x04,
    CapiControlModel = 0x05,
    EthernetNetworkingControlModel = 0x06,
    AtmNetworkingControlModel = 0x07,
    NetworkControlModel = 0x0D,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum CdcRequest {
    SetLineCoding = 0x20,
    GetLineCoding = 0x21,
    SetControlLineState = 0x22,
    SendBreak = 0x23,
}

impl TryFrom<u8> for CdcRequest {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x20 => Ok(Self::SetLineCoding),
            0x21 => Ok(Self::GetLineCoding),
            0x22 => Ok(Self::SetControlLineState),
            0x23 => Ok(Self::SendBreak),
            _ => Err(()),
        }
    }
}

/// CDC ACM Line Coding (7 bytes)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineCoding {
    pub baud_rate: u32,
    pub stop_bits: u8,
    pub parity: u8,
    pub data_bits: u8,
}

impl LineCoding {
    pub const SIZE: usize = 7;

    pub fn to_bytes(&self, bytes: &mut [u8; 7]) {
        LittleEndian::write_u32(&mut bytes[0..4], self.baud_rate);
        bytes[4] = self.stop_bits;
        bytes[5] = self.parity;
        bytes[6] = self.data_bits;
    }

    pub fn from_bytes(bytes: &[u8; 7]) -> Self {
        Self {
            baud_rate: LittleEndian::read_u32(&bytes[0..4]),
            stop_bits: bytes[4],
            parity: bytes[5],
            data_bits: bytes[6],
        }
    }
}

impl Default for LineCoding {
    fn default() -> Self {
        Self {
            baud_rate: 115200,
            stop_bits: 0, // 1 Stop bit
            parity: 0,    // None
            data_bits: 8, // 8 Data bits
        }
    }
}
