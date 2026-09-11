//! USB Human Interface Device (HID) definitions and reports.

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum HidSubclass {
    None = 0,
    BootInterface = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum HidProtocol {
    None = 0,
    Keyboard = 1,
    Mouse = 2,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum HidRequest {
    GetReport = 0x01,
    GetIdle = 0x02,
    GetProtocol = 0x03,
    SetReport = 0x09,
    SetIdle = 0x0A,
    SetProtocol = 0x0B,
}

impl TryFrom<u8> for HidRequest {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x01 => Ok(Self::GetReport),
            0x02 => Ok(Self::GetIdle),
            0x03 => Ok(Self::GetProtocol),
            0x09 => Ok(Self::SetReport),
            0x0A => Ok(Self::SetIdle),
            0x0B => Ok(Self::SetProtocol),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum HidReportType {
    Input = 1,
    Output = 2,
    Feature = 3,
}

/// Standard 8-byte HID Boot Keyboard Report
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct KeyboardReport {
    pub modifier: u8,
    pub reserved: u8,
    pub keycodes: [u8; 6],
}

impl KeyboardReport {
    pub fn to_bytes(&self, bytes: &mut [u8; 8]) {
        bytes[0] = self.modifier;
        bytes[1] = self.reserved;
        bytes[2..8].copy_from_slice(&self.keycodes);
    }

    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        let mut keycodes = [0u8; 6];
        keycodes.copy_from_slice(&bytes[2..8]);
        Self {
            modifier: bytes[0],
            reserved: bytes[1],
            keycodes,
        }
    }
}

/// Standard 4-byte HID Boot Mouse Report (buttons, X, Y, wheel)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct MouseReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
}

impl MouseReport {
    pub fn to_bytes(&self, bytes: &mut [u8; 4]) {
        bytes[0] = self.buttons;
        bytes[1] = self.x as u8;
        bytes[2] = self.y as u8;
        bytes[3] = self.wheel as u8;
    }

    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        Self {
            buttons: bytes[0],
            x: bytes[1] as i8,
            y: bytes[2] as i8,
            wheel: bytes[3] as i8,
        }
    }
}
