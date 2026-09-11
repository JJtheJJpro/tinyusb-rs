//! Core USB 2.0/3.0 & PD types, descriptors, setup packets, and standard requests.
#![no_std]

pub mod class;
pub mod descriptor;
pub mod setup;
pub mod types;

pub use class::*;
pub use descriptor::*;
pub use setup::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_packet_parsing() {
        // Standard GET_DESCRIPTOR Device (80 06 00 01 00 00 12 00)
        let raw = [0x80, 0x06, 0x00, 0x01, 0x00, 0x00, 0x12, 0x00];
        let setup = SetupPacket::from_bytes(&raw);

        assert_eq!(setup.direction(), Direction::In);
        assert_eq!(setup.req_type(), RequestType::Standard);
        assert_eq!(setup.recipient(), Recipient::Device);
        assert_eq!(setup.standard_request(), Some(StandardRequest::GetDescriptor));
        assert_eq!(setup.value, 0x0100);
        assert_eq!(setup.length, 18);

        let mut out = [0u8; 8];
        setup.to_bytes(&mut out);
        assert_eq!(raw, out);
    }

    #[test]
    fn test_device_descriptor_roundtrip() {
        let desc = DeviceDescriptor {
            bcd_usb: 0x0200,
            b_device_class: 0,
            b_device_sub_class: 0,
            b_device_protocol: 0,
            b_max_packet_size0: 64,
            id_vendor: 0x1234,
            id_product: 0x5678,
            bcd_device: 0x0100,
            i_manufacturer: 1,
            i_product: 2,
            i_serial_number: 3,
            b_num_configurations: 1,
        };

        let mut buf = [0u8; 18];
        desc.to_bytes(&mut buf);
        let parsed = DeviceDescriptor::from_bytes(&buf).expect("Failed to parse device descriptor");
        assert_eq!(desc, parsed);
    }

    #[test]
    fn test_speed_matches_tinyusb() {
        use crate::types::{Role, Speed};
        assert_eq!(Speed::Full as u8, 0);
        assert_eq!(Speed::Low as u8, 1);
        assert_eq!(Speed::High as u8, 2);
        assert_eq!(Speed::Auto as u8, 0xAA);
        assert_eq!(Speed::Invalid as u8, 0xFF);
        assert_eq!(Role::Invalid as u8, 0);
        assert_eq!(Role::Device as u8, 0x1);
        assert_eq!(Role::Host as u8, 0x2);
    }

    #[test]
    fn test_cbw_rejects_reserved_fields() {
        let mut raw = [0u8; 31];
        let valid = class::msc::CommandBlockWrapper {
            tag: 1,
            data_transfer_length: 0,
            flags: 0,
            lun: 0,
            cb_length: 6,
            command: [0; 16],
        };
        valid.to_bytes(&mut raw);
        assert!(class::msc::CommandBlockWrapper::from_bytes(&raw).is_some());

        raw[12] = 0x01; // reserved flag bits
        assert!(class::msc::CommandBlockWrapper::from_bytes(&raw).is_none());
    }

    #[test]
    fn test_cbw_csw_roundtrip() {
        let cbw = class::msc::CommandBlockWrapper {
            tag: 0x12345678,
            data_transfer_length: 512,
            flags: 0x80, // In
            lun: 0,
            cb_length: 10,
            command: [0x28, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], // READ(10)
        };

        let mut cbw_buf = [0u8; 31];
        cbw.to_bytes(&mut cbw_buf);
        let parsed_cbw = class::msc::CommandBlockWrapper::from_bytes(&cbw_buf).expect("Failed to parse CBW");
        assert_eq!(cbw, parsed_cbw);
        assert!(parsed_cbw.is_direction_in());

        let csw = class::msc::CommandStatusWrapper {
            tag: 0x12345678,
            data_residue: 0,
            status: class::msc::CommandStatus::Passed,
        };
        let mut csw_buf = [0u8; 13];
        csw.to_bytes(&mut csw_buf);
        let parsed_csw = class::msc::CommandStatusWrapper::from_bytes(&csw_buf).expect("Failed to parse CSW");
        assert_eq!(csw, parsed_csw);
    }
}
