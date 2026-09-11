//! Standard USB Descriptors.
use byteorder::{ByteOrder, LittleEndian};

/// USB Descriptor Types
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum DescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower = 0x08,
    Otg = 0x09,
    Debug = 0x0A,
    InterfaceAssociation = 0x0B,
    Bos = 0x0F,
    DeviceCapability = 0x10,
    Hid = 0x21,
    HidReport = 0x22,
    CsInterface = 0x24,
    CsEndpoint = 0x25,
}

/// USB Standard Device Descriptor (18 bytes)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct DeviceDescriptor {
    pub bcd_usb: u16,
    pub b_device_class: u8,
    pub b_device_sub_class: u8,
    pub b_device_protocol: u8,
    pub b_max_packet_size0: u8,
    pub id_vendor: u16,
    pub id_product: u16,
    pub bcd_device: u16,
    pub i_manufacturer: u8,
    pub i_product: u8,
    pub i_serial_number: u8,
    pub b_num_configurations: u8,
}

impl DeviceDescriptor {
    pub const SIZE: usize = 18;

    pub fn from_bytes(bytes: &[u8; 18]) -> Option<Self> {
        if bytes[0] != 18 || bytes[1] != DescriptorType::Device as u8 {
            return None;
        }
        Some(Self {
            bcd_usb: LittleEndian::read_u16(&bytes[2..4]),
            b_device_class: bytes[4],
            b_device_sub_class: bytes[5],
            b_device_protocol: bytes[6],
            b_max_packet_size0: bytes[7],
            id_vendor: LittleEndian::read_u16(&bytes[8..10]),
            id_product: LittleEndian::read_u16(&bytes[10..12]),
            bcd_device: LittleEndian::read_u16(&bytes[12..14]),
            i_manufacturer: bytes[14],
            i_product: bytes[15],
            i_serial_number: bytes[16],
            b_num_configurations: bytes[17],
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 18]) {
        bytes[0] = 18;
        bytes[1] = DescriptorType::Device as u8;
        LittleEndian::write_u16(&mut bytes[2..4], self.bcd_usb);
        bytes[4] = self.b_device_class;
        bytes[5] = self.b_device_sub_class;
        bytes[6] = self.b_device_protocol;
        bytes[7] = self.b_max_packet_size0;
        LittleEndian::write_u16(&mut bytes[8..10], self.id_vendor);
        LittleEndian::write_u16(&mut bytes[10..12], self.id_product);
        LittleEndian::write_u16(&mut bytes[12..14], self.bcd_device);
        bytes[14] = self.i_manufacturer;
        bytes[15] = self.i_product;
        bytes[16] = self.i_serial_number;
        bytes[17] = self.b_num_configurations;
    }
}

/// USB Configuration Descriptor (9 bytes)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConfigurationDescriptor {
    pub w_total_length: u16,
    pub b_num_interfaces: u8,
    pub b_configuration_value: u8,
    pub i_configuration: u8,
    pub bm_attributes: u8,
    pub b_max_power: u8, // in 2mA units (USB 2.0)
}

impl ConfigurationDescriptor {
    pub const SIZE: usize = 9;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 9 || bytes[0] != 9 || bytes[1] != DescriptorType::Configuration as u8 {
            return None;
        }
        Some(Self {
            w_total_length: LittleEndian::read_u16(&bytes[2..4]),
            b_num_interfaces: bytes[4],
            b_configuration_value: bytes[5],
            i_configuration: bytes[6],
            bm_attributes: bytes[7],
            b_max_power: bytes[8],
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 9]) {
        bytes[0] = 9;
        bytes[1] = DescriptorType::Configuration as u8;
        LittleEndian::write_u16(&mut bytes[2..4], self.w_total_length);
        bytes[4] = self.b_num_interfaces;
        bytes[5] = self.b_configuration_value;
        bytes[6] = self.i_configuration;
        bytes[7] = self.bm_attributes;
        bytes[8] = self.b_max_power;
    }
}

/// USB Interface Descriptor (9 bytes)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InterfaceDescriptor {
    pub b_interface_number: u8,
    pub b_alternate_setting: u8,
    pub b_num_endpoints: u8,
    pub b_interface_class: u8,
    pub b_interface_sub_class: u8,
    pub b_interface_protocol: u8,
    pub i_interface: u8,
}

impl InterfaceDescriptor {
    pub const SIZE: usize = 9;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 9 || bytes[0] != 9 || bytes[1] != DescriptorType::Interface as u8 {
            return None;
        }
        Some(Self {
            b_interface_number: bytes[2],
            b_alternate_setting: bytes[3],
            b_num_endpoints: bytes[4],
            b_interface_class: bytes[5],
            b_interface_sub_class: bytes[6],
            b_interface_protocol: bytes[7],
            i_interface: bytes[8],
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 9]) {
        bytes[0] = 9;
        bytes[1] = DescriptorType::Interface as u8;
        bytes[2] = self.b_interface_number;
        bytes[3] = self.b_alternate_setting;
        bytes[4] = self.b_num_endpoints;
        bytes[5] = self.b_interface_class;
        bytes[6] = self.b_interface_sub_class;
        bytes[7] = self.b_interface_protocol;
        bytes[8] = self.i_interface;
    }
}

/// USB Endpoint Descriptor (7 bytes)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct EndpointDescriptor {
    pub b_endpoint_address: u8,
    pub bm_attributes: u8,
    pub w_max_packet_size: u16,
    pub b_interval: u8,
}

impl EndpointDescriptor {
    pub const SIZE: usize = 7;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 7 || bytes[0] != 7 || bytes[1] != DescriptorType::Endpoint as u8 {
            return None;
        }
        Some(Self {
            b_endpoint_address: bytes[2],
            bm_attributes: bytes[3],
            w_max_packet_size: LittleEndian::read_u16(&bytes[4..6]),
            b_interval: bytes[6],
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 7]) {
        bytes[0] = 7;
        bytes[1] = DescriptorType::Endpoint as u8;
        bytes[2] = self.b_endpoint_address;
        bytes[3] = self.bm_attributes;
        LittleEndian::write_u16(&mut bytes[4..6], self.w_max_packet_size);
        bytes[6] = self.b_interval;
    }
}

/// Interface Association Descriptor (IAD, 8 bytes)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InterfaceAssociationDescriptor {
    pub b_first_interface: u8,
    pub b_interface_count: u8,
    pub b_function_class: u8,
    pub b_function_sub_class: u8,
    pub b_function_protocol: u8,
    pub i_function: u8,
}

impl InterfaceAssociationDescriptor {
    pub const SIZE: usize = 8;

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 || bytes[0] != 8 || bytes[1] != DescriptorType::InterfaceAssociation as u8 {
            return None;
        }
        Some(Self {
            b_first_interface: bytes[2],
            b_interface_count: bytes[3],
            b_function_class: bytes[4],
            b_function_sub_class: bytes[5],
            b_function_protocol: bytes[6],
            i_function: bytes[7],
        })
    }

    pub fn to_bytes(&self, bytes: &mut [u8; 8]) {
        bytes[0] = 8;
        bytes[1] = DescriptorType::InterfaceAssociation as u8;
        bytes[2] = self.b_first_interface;
        bytes[3] = self.b_interface_count;
        bytes[4] = self.b_function_class;
        bytes[5] = self.b_function_sub_class;
        bytes[6] = self.b_function_protocol;
        bytes[7] = self.i_function;
    }
}
