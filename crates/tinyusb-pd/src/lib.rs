//! USB Type-C and Power Delivery (PD) protocol definitions and state machine.
#![no_std]

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PortType {
    Source = 0,
    Sink = 1,
    DualRolePower = 2,
    Disconnected = 3,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum DataRole {
    UpstreamFacingPort = 0,
    DownstreamFacingPort = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PowerRole {
    Sink = 0,
    Source = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PdControlMessage {
    GoodCrc = 1,
    GotoMin = 2,
    Accept = 3,
    Reject = 4,
    Ping = 5,
    PsReady = 6,
    GetSourceCap = 7,
    GetSinkCap = 8,
    DrSwap = 9,
    PrSwap = 10,
    VconnSwap = 11,
    Wait = 12,
    SoftReset = 13,
    NotSupported = 16,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum PdDataMessage {
    SourceCapabilities = 1,
    Request = 2,
    SinkCapabilities = 4,
    BatteryStatus = 5,
    Alert = 6,
    VendorDefined = 15,
}

/// 16-bit USB PD Message Header
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MessageHeader(pub u16);

impl MessageHeader {
    pub fn new(
        msg_type: u8,
        data_role: DataRole,
        power_role: PowerRole,
        msg_id: u8,
        num_data_objects: u8,
    ) -> Self {
        let val = (msg_type as u16 & 0x1F)
            | ((data_role as u16 & 0x01) << 5)
            | (0x02 << 6) // Revision 3.0 (0b10)
            | ((power_role as u16 & 0x01) << 8)
            | ((msg_id as u16 & 0x07) << 9)
            | ((num_data_objects as u16 & 0x07) << 12);
        Self(val)
    }

    pub fn msg_type(&self) -> u8 {
        (self.0 & 0x1F) as u8
    }

    pub fn data_role(&self) -> DataRole {
        if (self.0 & (1 << 5)) != 0 {
            DataRole::DownstreamFacingPort
        } else {
            DataRole::UpstreamFacingPort
        }
    }

    pub fn power_role(&self) -> PowerRole {
        if (self.0 & (1 << 8)) != 0 {
            PowerRole::Source
        } else {
            PowerRole::Sink
        }
    }

    pub fn msg_id(&self) -> u8 {
        ((self.0 >> 9) & 0x07) as u8
    }

    pub fn num_data_objects(&self) -> u8 {
        ((self.0 >> 12) & 0x07) as u8
    }

    pub fn is_control_message(&self) -> bool {
        self.num_data_objects() == 0
    }

    pub fn is_data_message(&self) -> bool {
        self.num_data_objects() > 0
    }
}

/// Fixed Power Data Object (Source)
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct FixedSupplyPdo(pub u32);

impl FixedSupplyPdo {
    pub fn new(voltage_50mv: u16, max_current_10ma: u16) -> Self {
        // [31:30] = 00b (Fixed supply)
        let val = (max_current_10ma as u32 & 0x3FF) | ((voltage_50mv as u32 & 0x3FF) << 10);
        Self(val)
    }

    pub fn voltage_mv(&self) -> u32 {
        ((self.0 >> 10) & 0x3FF) * 50
    }

    pub fn max_current_ma(&self) -> u32 {
        (self.0 & 0x3FF) * 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header() {
        let hdr = MessageHeader::new(
            PdControlMessage::GoodCrc as u8,
            DataRole::UpstreamFacingPort,
            PowerRole::Sink,
            3,
            0,
        );

        assert_eq!(hdr.msg_type(), PdControlMessage::GoodCrc as u8);
        assert_eq!(hdr.data_role(), DataRole::UpstreamFacingPort);
        assert_eq!(hdr.power_role(), PowerRole::Sink);
        assert_eq!(hdr.msg_id(), 3);
        assert_eq!(hdr.num_data_objects(), 0);
        assert!(hdr.is_control_message());
        assert!(!hdr.is_data_message());
    }

    #[test]
    fn test_fixed_supply_pdo() {
        // 5V (100 * 50mV), 3A (300 * 10mA)
        let pdo = FixedSupplyPdo::new(100, 300);
        assert_eq!(pdo.voltage_mv(), 5000);
        assert_eq!(pdo.max_current_ma(), 3000);
    }
}
