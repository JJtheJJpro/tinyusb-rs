//! USB Control Transfer Setup Packet and Standard Requests.
use crate::types::Direction;
use byteorder::{ByteOrder, LittleEndian};

/// Request Type: Standard, Class, Vendor, or Reserved.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum RequestType {
    Standard = 0,
    Class = 1,
    Vendor = 2,
    Reserved = 3,
}

/// Request Recipient: Device, Interface, Endpoint, or Other.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Recipient {
    Device = 0,
    Interface = 1,
    Endpoint = 2,
    Other = 3,
}

/// USB Standard Requests (USB 2.0 Spec Table 9-4)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum StandardRequest {
    GetStatus = 0,
    ClearFeature = 1,
    SetFeature = 3,
    SetAddress = 5,
    GetDescriptor = 6,
    SetDescriptor = 7,
    GetConfiguration = 8,
    SetConfiguration = 9,
    GetInterface = 10,
    SetInterface = 11,
    SynchFrame = 12,
}

impl TryFrom<u8> for StandardRequest {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::GetStatus),
            1 => Ok(Self::ClearFeature),
            3 => Ok(Self::SetFeature),
            5 => Ok(Self::SetAddress),
            6 => Ok(Self::GetDescriptor),
            7 => Ok(Self::SetDescriptor),
            8 => Ok(Self::GetConfiguration),
            9 => Ok(Self::SetConfiguration),
            10 => Ok(Self::GetInterface),
            11 => Ok(Self::SetInterface),
            12 => Ok(Self::SynchFrame),
            _ => Err(()),
        }
    }
}

/// 8-byte USB Control Transfer Setup Packet
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SetupPacket {
    pub request_type: u8,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupPacket {
    pub const SIZE: usize = 8;

    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            request_type: bytes[0],
            request: bytes[1],
            value: LittleEndian::read_u16(&bytes[2..4]),
            index: LittleEndian::read_u16(&bytes[4..6]),
            length: LittleEndian::read_u16(&bytes[6..8]),
        }
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 8]) {
        bytes[0] = self.request_type;
        bytes[1] = self.request;
        LittleEndian::write_u16(&mut bytes[2..4], self.value);
        LittleEndian::write_u16(&mut bytes[4..6], self.index);
        LittleEndian::write_u16(&mut bytes[6..8], self.length);
    }

    pub fn direction(&self) -> Direction {
        if (self.request_type & 0x80) != 0 {
            Direction::In
        } else {
            Direction::Out
        }
    }

    pub fn req_type(&self) -> RequestType {
        match (self.request_type >> 5) & 0x03 {
            0 => RequestType::Standard,
            1 => RequestType::Class,
            2 => RequestType::Vendor,
            _ => RequestType::Reserved,
        }
    }

    pub fn recipient(&self) -> Recipient {
        match self.request_type & 0x1F {
            0 => Recipient::Device,
            1 => Recipient::Interface,
            2 => Recipient::Endpoint,
            _ => Recipient::Other,
        }
    }

    pub fn standard_request(&self) -> Option<StandardRequest> {
        if self.req_type() == RequestType::Standard {
            StandardRequest::try_from(self.request).ok()
        } else {
            None
        }
    }
}
