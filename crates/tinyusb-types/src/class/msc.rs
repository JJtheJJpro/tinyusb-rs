//! USB Mass Storage Class (MSC) BOT and SCSI definitions.
use byteorder::{ByteOrder, LittleEndian};

pub const CBW_SIGNATURE: u32 = 0x4342_5355; // 'USBC'
pub const CSW_SIGNATURE: u32 = 0x5342_5355; // 'USBS'

pub const CBW_SIZE: usize = 31;
pub const CSW_SIZE: usize = 13;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum MscSubclass {
    Rbc = 1,
    SffMmc = 2,
    Qic = 3,
    Ufi = 4,
    Sff = 5,
    Scsi = 6,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum MscProtocol {
    CbiWithInterrupt = 0,
    CbiNoInterrupt = 1,
    BulkOnly = 0x50,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum CommandStatus {
    Passed = 0,
    Failed = 1,
    PhaseError = 2,
}

/// Command Block Wrapper (CBW, 31 bytes)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandBlockWrapper {
    pub tag: u32,
    pub data_transfer_length: u32,
    pub flags: u8, // Bit 7: 1 = In (device to host), 0 = Out (host to device)
    pub lun: u8,
    pub cb_length: u8,
    pub command: [u8; 16],
}

impl CommandBlockWrapper {
    pub fn is_direction_in(&self) -> bool {
        (self.flags & 0x80) != 0
    }

    pub fn is_direction_out(&self) -> bool {
        (self.flags & 0x80) == 0
    }

    pub fn from_bytes(bytes: &[u8; 31]) -> Option<Self> {
        let sig = LittleEndian::read_u32(&bytes[0..4]);
        if sig != CBW_SIGNATURE {
            return None;
        }

        // BOT: direction is bit 7 only; LUN is lower 4 bits; CDB length is lower 5 bits.
        if (bytes[12] & 0x7F) != 0 || (bytes[13] & 0xF0) != 0 || (bytes[14] & 0xE0) != 0 {
            return None;
        }

        let cb_length = bytes[14] & 0x1F;
        if !(1..=16).contains(&cb_length) {
            return None;
        }

        let mut command = [0u8; 16];
        command.copy_from_slice(&bytes[15..31]);

        Some(Self {
            tag: LittleEndian::read_u32(&bytes[4..8]),
            data_transfer_length: LittleEndian::read_u32(&bytes[8..12]),
            flags: bytes[12],
            lun: bytes[13] & 0x0F,
            cb_length,
            command,
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 31]) {
        LittleEndian::write_u32(&mut bytes[0..4], CBW_SIGNATURE);
        LittleEndian::write_u32(&mut bytes[4..8], self.tag);
        LittleEndian::write_u32(&mut bytes[8..12], self.data_transfer_length);
        bytes[12] = self.flags;
        bytes[13] = self.lun & 0x0F;
        bytes[14] = self.cb_length & 0x1F;
        bytes[15..31].copy_from_slice(&self.command);
    }
}

/// Command Status Wrapper (CSW, 13 bytes)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommandStatusWrapper {
    pub tag: u32,
    pub data_residue: u32,
    pub status: CommandStatus,
}

impl CommandStatusWrapper {
    pub fn to_bytes(&self, bytes: &mut [u8; 13]) {
        LittleEndian::write_u32(&mut bytes[0..4], CSW_SIGNATURE);
        LittleEndian::write_u32(&mut bytes[4..8], self.tag);
        LittleEndian::write_u32(&mut bytes[8..12], self.data_residue);
        bytes[12] = self.status as u8;
    }

    pub fn from_bytes(bytes: &[u8; 13]) -> Option<Self> {
        let sig = LittleEndian::read_u32(&bytes[0..4]);
        if sig != CSW_SIGNATURE {
            return None;
        }
        let status = match bytes[12] {
            0 => CommandStatus::Passed,
            1 => CommandStatus::Failed,
            2 => CommandStatus::PhaseError,
            _ => return None,
        };

        Some(Self {
            tag: LittleEndian::read_u32(&bytes[4..8]),
            data_residue: LittleEndian::read_u32(&bytes[8..12]),
            status,
        })
    }
}

/// SCSI Command Opcodes
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum ScsiCommand {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ModeSelect6 = 0x15,
    ModeSense6 = 0x1A,
    StartStopUnit = 0x1B,
    PreventAllowMediumRemoval = 0x1E,
    ReadFormatCapacities = 0x23,
    ReadCapacity10 = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
}

impl TryFrom<u8> for ScsiCommand {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x00 => Ok(Self::TestUnitReady),
            0x03 => Ok(Self::RequestSense),
            0x12 => Ok(Self::Inquiry),
            0x15 => Ok(Self::ModeSelect6),
            0x1A => Ok(Self::ModeSense6),
            0x1B => Ok(Self::StartStopUnit),
            0x1E => Ok(Self::PreventAllowMediumRemoval),
            0x23 => Ok(Self::ReadFormatCapacities),
            0x25 => Ok(Self::ReadCapacity10),
            0x28 => Ok(Self::Read10),
            0x2A => Ok(Self::Write10),
            _ => Err(()),
        }
    }
}

/// SCSI Sense Keys
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum ScsiSenseKey {
    None = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
    AbortedCommand = 0x0B,
}
