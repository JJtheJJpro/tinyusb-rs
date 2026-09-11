//! USB primitive types and constants.
use bitflags::bitflags;

/// USB Speed (matches TinyUSB `tusb_speed_t` / EHCI encoding).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(u8)]
pub enum Speed {
    Full = 0,
    Low = 1,
    High = 2,
    Auto = 0xAA,
    Invalid = 0xFF,
}

/// USB Role (matches TinyUSB `tusb_role_t`).
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Role {
    Invalid = 0,
    Device = 0x1,
    Host = 0x2,
}

/// USB Transfer Type
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum TransferType {
    Control = 0,
    Isochronous = 1,
    Bulk = 2,
    Interrupt = 3,
}

/// USB Transfer Direction
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Direction {
    Out = 0,
    In = 1,
}

/// USB Endpoint Address with Direction
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EndpointAddress(pub u8);

impl EndpointAddress {
    pub const fn from_parts(number: u8, dir: Direction) -> Self {
        let addr = (number & 0x0F) | match dir {
            Direction::In => 0x80,
            Direction::Out => 0x00,
        };
        Self(addr)
    }

    pub const fn number(&self) -> u8 {
        self.0 & 0x0F
    }

    pub const fn direction(&self) -> Direction {
        if (self.0 & 0x80) != 0 {
            Direction::In
        } else {
            Direction::Out
        }
    }

    pub const fn is_in(&self) -> bool {
        (self.0 & 0x80) != 0
    }

    pub const fn is_out(&self) -> bool {
        (self.0 & 0x80) == 0
    }
}

bitflags! {
    /// Isochronous Endpoint Synchronization and Usage Attributes
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct IsoAttributes: u8 {
        const NO_SYNC = 0x00;
        const ASYNCHRONOUS = 0x04;
        const ADAPTIVE = 0x08;
        const SYNCHRONOUS = 0x0C;
        const USAGE_DATA = 0x00;
        const USAGE_FEEDBACK = 0x10;
        const USAGE_IMPLICIT_FEEDBACK = 0x20;
    }
}

/// USB Class Codes
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum ClassCode {
    Unspecified = 0x00,
    Audio = 0x01,
    Cdc = 0x02,
    Hid = 0x03,
    Physical = 0x05,
    Image = 0x06,
    Printer = 0x07,
    MassStorage = 0x08,
    Hub = 0x09,
    CdcData = 0x0A,
    SmartCard = 0x0B,
    ContentSecurity = 0x0D,
    Video = 0x0E,
    PersonalHealthcare = 0x0F,
    AudioVideo = 0x10,
    Billboard = 0x11,
    TypeCBridge = 0x12,
    Diagnostic = 0xDC,
    WirelessController = 0xE0,
    Miscellaneous = 0xEF,
    ApplicationSpecific = 0xFE,
    VendorSpecific = 0xFF,
}
