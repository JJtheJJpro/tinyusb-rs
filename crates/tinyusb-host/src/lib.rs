//! Async USB Host framework for embassy-rs.
#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Timer};
use heapless::Vec;
use tinyusb_types::descriptor::{DeviceDescriptor, EndpointDescriptor};
use tinyusb_types::setup::SetupPacket;
use tinyusb_types::types::{Speed, TransferType};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum HostError {
    BusError,
    Timeout,
    Babble,
    Stall,
    DeviceDisconnected,
    NoChannelAvailable,
    DescriptorParseError,
    BufferOverflow,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PortStatus {
    pub connected: bool,
    pub speed: Speed,
    pub enabled: bool,
    pub suspended: bool,
}

/// Abstract Pipe representing an allocated channel to a specific endpoint on an attached device.
pub trait Pipe {
    /// Submit a transfer to the pipe and await completion.
    async fn transfer(&mut self, setup: Option<&SetupPacket>, buf: &mut [u8]) -> Result<usize, HostError>;
}

/// Host Controller Driver trait.
/// Hardware controllers (e.g. Synopsys DWC2, RP2040 host, MAX3421E) implement this.
pub trait HostDriver {
    type Pipe<'a>: Pipe where Self: 'a;

    /// Read the port status of a root hub port.
    async fn port_status(&mut self, port: u8) -> PortStatus;

    /// Reset a root hub port.
    async fn reset_port(&mut self, port: u8) -> Result<Speed, HostError>;

    /// Open a communication pipe to a specific endpoint on an attached device.
    fn open_pipe<'a>(
        &'a mut self,
        dev_addr: u8,
        ep_desc: &EndpointDescriptor,
    ) -> Result<Self::Pipe<'a>, HostError>;
}

/// Information about an enumerated USB device.
#[derive(Clone, Debug)]
pub struct AttachedDevice {
    pub address: u8,
    pub speed: Speed,
    pub device_desc: DeviceDescriptor,
}

/// Enumeration engine for USB Host.
pub struct UsbHost<D: HostDriver, const MAX_DEVICES: usize = 4> {
    driver: D,
    next_address: u8,
    devices: Vec<AttachedDevice, MAX_DEVICES>,
}

impl<D: HostDriver, const MAX_DEVICES: usize> UsbHost<D, MAX_DEVICES> {
    pub fn new(driver: D) -> Self {
        Self {
            driver,
            next_address: 1,
            devices: Vec::new(),
        }
    }

    /// Monitor root port and enumerate newly attached devices.
    pub async fn run_enumeration(&mut self, port: u8) -> Result<AttachedDevice, HostError> {
        // 1. Wait for connection
        loop {
            let status = self.driver.port_status(port).await;
            if status.connected {
                break;
            }
            Timer::after(Duration::from_millis(50)).await;
        }

        // 2. Debounce connection (100ms standard USB debounce)
        Timer::after(Duration::from_millis(100)).await;

        // 3. Reset port
        let speed = self.driver.reset_port(port).await?;
        Timer::after(Duration::from_millis(20)).await; // Reset recovery time

        // 4. Default control endpoint (EP0, Address 0)
        let ep0_desc = EndpointDescriptor {
            b_endpoint_address: 0,
            bm_attributes: TransferType::Control as u8,
            w_max_packet_size: if speed == Speed::Low { 8 } else { 64 },
            b_interval: 0,
        };

        let mut ep0 = self.driver.open_pipe(0, &ep0_desc)?;

        // 5. Read initial 8 bytes of device descriptor to learn EP0 max packet size
        let mut desc_buf = [0u8; 18];
        let setup_get_desc = SetupPacket {
            request_type: 0x80, // In, Standard, Device
            request: 6,         // GET_DESCRIPTOR
            value: 0x0100,      // Device
            index: 0,
            length: 8,
        };
        ep0.transfer(Some(&setup_get_desc), &mut desc_buf[..8]).await?;

        let max_packet_size0 = desc_buf[7];

        // 6. Assign unique device address
        let new_addr = self.next_address;
        self.next_address = if self.next_address >= 127 {
            1
        } else {
            self.next_address + 1
        };

        let setup_set_addr = SetupPacket {
            request_type: 0x00, // Out, Standard, Device
            request: 5,         // SET_ADDRESS
            value: new_addr as u16,
            index: 0,
            length: 0,
        };
        ep0.transfer(Some(&setup_set_addr), &mut []).await?;
        Timer::after(Duration::from_millis(10)).await; // SetAddress recovery time

        // 7. Re-open EP0 with new address and confirmed max packet size
        drop(ep0);
        let ep0_desc_final = EndpointDescriptor {
            b_endpoint_address: 0,
            bm_attributes: TransferType::Control as u8,
            w_max_packet_size: max_packet_size0 as u16,
            b_interval: 0,
        };
        let mut ep0_addressed = self.driver.open_pipe(new_addr, &ep0_desc_final)?;

        // 8. Read full 18-byte device descriptor
        let setup_full_desc = SetupPacket {
            request_type: 0x80,
            request: 6,
            value: 0x0100,
            index: 0,
            length: 18,
        };
        ep0_addressed.transfer(Some(&setup_full_desc), &mut desc_buf).await?;

        let device_desc = DeviceDescriptor::from_bytes(&desc_buf)
            .ok_or(HostError::DescriptorParseError)?;

        let attached = AttachedDevice {
            address: new_addr,
            speed,
            device_desc,
        };

        let _ = self.devices.push(attached.clone());
        Ok(attached)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[no_mangle]
    extern "Rust" fn _embassy_time_now() -> u64 {
        0
    }

    #[no_mangle]
    extern "Rust" fn _embassy_time_schedule_wake(_at: u64, _waker: &core::task::Waker) {}

    #[test]
    fn test_port_status_creation() {
        let status = PortStatus {
            connected: true,
            speed: Speed::High,
            enabled: true,
            suspended: false,
        };
        assert!(status.connected);
        assert_eq!(status.speed, Speed::High);
    }
}
