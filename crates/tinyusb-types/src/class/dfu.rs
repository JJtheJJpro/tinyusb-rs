//! USB Device Firmware Upgrade (DFU 1.1) definitions.
use byteorder::{ByteOrder, LittleEndian};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum DfuRequest {
    Detach = 0,
    Dnload = 1,
    Upload = 2,
    GetStatus = 3,
    ClrStatus = 4,
    GetState = 5,
    Abort = 6,
}

impl TryFrom<u8> for DfuRequest {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Detach),
            1 => Ok(Self::Dnload),
            2 => Ok(Self::Upload),
            3 => Ok(Self::GetStatus),
            4 => Ok(Self::ClrStatus),
            5 => Ok(Self::GetState),
            6 => Ok(Self::Abort),
            _ => Err(()),
        }
    }
}

/// DFU Functional Descriptor (9 bytes)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DfuFunctionalDescriptor {
    pub bm_attributes: u8,
    pub w_detach_time_out: u16,
    pub w_transfer_size: u16,
    pub bcd_dfu_version: u16,
}

impl DfuFunctionalDescriptor {
    pub const SIZE: usize = 9;

    pub fn to_bytes(&self, bytes: &mut [u8; 9]) {
        bytes[0] = 9;
        bytes[1] = 0x21; // DFU Functional Descriptor type
        bytes[2] = self.bm_attributes;
        LittleEndian::write_u16(&mut bytes[3..5], self.w_detach_time_out);
        LittleEndian::write_u16(&mut bytes[5..7], self.w_transfer_size);
        LittleEndian::write_u16(&mut bytes[7..9], self.bcd_dfu_version);
    }

    pub fn from_bytes(bytes: &[u8; 9]) -> Option<Self> {
        if bytes[0] != 9 || bytes[1] != 0x21 {
            return None;
        }
        Some(Self {
            bm_attributes: bytes[2],
            w_detach_time_out: LittleEndian::read_u16(&bytes[3..5]),
            w_transfer_size: LittleEndian::read_u16(&bytes[5..7]),
            bcd_dfu_version: LittleEndian::read_u16(&bytes[7..9]),
        })
    }
}
