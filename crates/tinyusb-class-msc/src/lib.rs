//! USB Mass Storage Class (MSC) for embassy-usb.
#![no_std]

pub mod scsi;

use byteorder::{BigEndian, ByteOrder};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Driver, EndpointIn, EndpointOut};
use embassy_usb::{Builder, Handler};
use scsi::{BlockDevice, ScsiHandler};
use tinyusb_types::class::msc::{
    CommandBlockWrapper, CommandStatus, CommandStatusWrapper, ScsiCommand, ScsiSenseKey,
};

const MSC_REQ_GET_MAX_LUN: u8 = 0xFE;
const MSC_REQ_BOT_RESET: u8 = 0xFF;

/// Cap IN data length by CBW transfer length and the SCSI CDB allocation length (byte 4).
fn clamp_scsi_in_length(cbw: &CommandBlockWrapper, response_len: usize) -> usize {
    let mut len = response_len.min(cbw.data_transfer_length as usize);
    let alloc = cbw.command[4];
    if alloc != 0 {
        len = len.min(alloc as usize);
    }
    len
}

/// BOT validation for READ(10) / WRITE(10) (TinyUSB `rdwr10_validate_cmd`).
fn validate_rdwr10(cbw: &CommandBlockWrapper, is_read: bool) -> Result<(u32, u32), CommandStatus> {
    let block_count = BigEndian::read_u16(&cbw.command[7..9]) as u32;
    if cbw.data_transfer_length == 0 {
        if block_count > 0 {
            return Err(CommandStatus::PhaseError);
        }
        return Ok((0, 0));
    }
    if is_read && cbw.is_direction_out() {
        return Err(CommandStatus::PhaseError);
    }
    if !is_read && cbw.is_direction_in() {
        return Err(CommandStatus::PhaseError);
    }
    if block_count == 0 {
        return Err(CommandStatus::Failed);
    }
    let block_size = cbw.data_transfer_length / block_count;
    if block_size == 0 {
        return Err(CommandStatus::PhaseError);
    }
    Ok((block_count, block_size))
}

/// Embassy USB Control Handler for MSC (handles GET_MAX_LUN and BOT_RESET)
pub struct MscControlHandler {
    max_lun: u8,
}

impl MscControlHandler {
    /// `lun_count` is the number of LUNs (1–16), same as TinyUSB `tud_msc_get_maxlun_cb()`.
    /// GET_MAX_LUN returns `lun_count - 1` per the BOT spec.
    pub fn new(lun_count: u8) -> Self {
        debug_assert!(lun_count > 0, "MSC must expose at least one LUN");
        Self {
            max_lun: lun_count.saturating_sub(1),
        }
    }
}

impl Handler for MscControlHandler {
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        if req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.request == MSC_REQ_GET_MAX_LUN
        {
            if !buf.is_empty() {
                buf[0] = self.max_lun;
                return Some(InResponse::Accepted(&buf[..1]));
            }
        }
        None
    }

    fn control_out(&mut self, req: Request, _buf: &[u8]) -> Option<OutResponse> {
        if req.request_type == RequestType::Class
            && req.recipient == Recipient::Interface
            && req.request == MSC_REQ_BOT_RESET
        {
            return Some(OutResponse::Accepted);
        }
        None
    }
}

/// MSC Class wrapper for embassy-usb
pub struct MscClass<'d, D: Driver<'d>> {
    ep_in: D::EndpointIn,
    ep_out: D::EndpointOut,
}

impl<'d, D: Driver<'d>> MscClass<'d, D> {
    pub const MAX_PACKET_SIZE: u16 = 64;

    pub fn new(builder: &mut Builder<'d, D>, max_packet_size: u16) -> Self {
        let mut func = builder.function(0x08, 0x06, 0x50); // Mass Storage, SCSI transparent, Bulk-Only
        let mut iface = func.interface();
        let mut alt = iface.alt_setting(0x08, 0x06, 0x50, None);

        let ep_out = alt.endpoint_bulk_out(None, max_packet_size);
        let ep_in = alt.endpoint_bulk_in(None, max_packet_size);

        Self { ep_in, ep_out }
    }

    /// Run the Mass Storage BOT engine with a backing block device.
    pub async fn run<B: BlockDevice>(
        mut self,
        dev: &mut B,
        vendor: &[u8; 8],
        product: &[u8; 16],
        revision: &[u8; 4],
    ) -> ! {
        let mut scsi = ScsiHandler::new();
        let mut cbw_buf = [0u8; 31];
        let mut csw_buf = [0u8; 13];
        let mut sector_buf = [0u8; 512];

        loop {
            // 1. Read CBW from host
            if self.ep_out.read(&mut cbw_buf).await.is_err() {
                continue;
            }

            let cbw = match CommandBlockWrapper::from_bytes(&cbw_buf) {
                Some(c) => c,
                None => {
                    // Invalid CBW, stall endpoints
                    continue;
                }
            };

            let opcode = cbw.command[0];
            let scsi_cmd = ScsiCommand::try_from(opcode);

            let mut status;
            let mut residue = cbw.data_transfer_length;

            match scsi_cmd {
                Ok(ScsiCommand::TestUnitReady) => {
                    residue = 0;
                    status = CommandStatus::Passed;
                }
                Ok(ScsiCommand::Inquiry) => {
                    let mut inq_buf = [0u8; 36];
                    let len = scsi.handle_inquiry(vendor, product, revision, &mut inq_buf);
                    let to_send = clamp_scsi_in_length(&cbw, len);
                    if to_send == 0 || self.ep_in.write(&inq_buf[..to_send]).await.is_ok() {
                        residue = cbw.data_transfer_length.saturating_sub(to_send as u32);
                        status = CommandStatus::Passed;
                    } else {
                        status = CommandStatus::Failed;
                    }
                }
                Ok(ScsiCommand::ReadCapacity10) => {
                    let mut cap_buf = [0u8; 8];
                    let len = scsi.handle_read_capacity_10(dev, &mut cap_buf);
                    if len == 0 {
                        status = CommandStatus::Failed;
                    } else {
                        let to_send = clamp_scsi_in_length(&cbw, len);
                        if self.ep_in.write(&cap_buf[..to_send]).await.is_ok() {
                            residue = cbw.data_transfer_length.saturating_sub(to_send as u32);
                            status = CommandStatus::Passed;
                        } else {
                            status = CommandStatus::Failed;
                        }
                    }
                }
                Ok(ScsiCommand::RequestSense) => {
                    let mut sense_buf = [0u8; 18];
                    let len = scsi.handle_request_sense(&mut sense_buf);
                    let to_send = clamp_scsi_in_length(&cbw, len);
                    if to_send == 0 || self.ep_in.write(&sense_buf[..to_send]).await.is_ok() {
                        residue = cbw.data_transfer_length.saturating_sub(to_send as u32);
                        status = CommandStatus::Passed;
                    } else {
                        status = CommandStatus::Failed;
                    }
                }
                Ok(ScsiCommand::ModeSense6) => {
                    let mut mode_buf = [0u8; 4];
                    let len = scsi.handle_mode_sense_6(dev.is_writable(), &mut mode_buf);
                    let to_send = clamp_scsi_in_length(&cbw, len);
                    if self.ep_in.write(&mode_buf[..to_send]).await.is_ok() {
                        residue = cbw.data_transfer_length.saturating_sub(to_send as u32);
                        status = CommandStatus::Passed;
                    } else {
                        status = CommandStatus::Failed;
                    }
                }
                Ok(ScsiCommand::Read10) => {
                    status = CommandStatus::Passed;
                    match validate_rdwr10(&cbw, true) {
                        Ok((blocks, _block_size)) if blocks == 0 => {
                            residue = 0;
                        }
                        Ok((blocks, block_size)) => {
                            if block_size != dev.block_size() || block_size as usize > sector_buf.len() {
                                scsi.set_sense(ScsiSenseKey::IllegalRequest, 0x20, 0x00);
                                status = CommandStatus::Failed;
                            } else {
                                let lba = BigEndian::read_u32(&cbw.command[2..6]);
                                let mut xferred = 0u32;
                                let mut read_err = false;
                                for b in 0..blocks {
                                    if dev.read_block(lba + b, &mut sector_buf).is_ok() {
                                        if self
                                            .ep_in
                                            .write(&sector_buf[..block_size as usize])
                                            .await
                                            .is_err()
                                        {
                                            read_err = true;
                                            break;
                                        }
                                        xferred += block_size;
                                    } else {
                                        read_err = true;
                                        break;
                                    }
                                }
                                residue = cbw.data_transfer_length.saturating_sub(xferred);
                                if read_err {
                                    scsi.set_sense(ScsiSenseKey::MediumError, 0x11, 0x00);
                                    status = CommandStatus::Failed;
                                }
                            }
                        }
                        Err(st) => status = st,
                    }
                }
                Ok(ScsiCommand::Write10) => {
                    status = CommandStatus::Passed;
                    if !dev.is_writable() {
                        scsi.set_sense(ScsiSenseKey::DataProtect, 0x27, 0x00);
                        status = CommandStatus::Failed;
                    } else {
                        match validate_rdwr10(&cbw, false) {
                            Ok((blocks, _block_size)) if blocks == 0 => {
                                residue = 0;
                            }
                            Ok((blocks, block_size)) => {
                                if block_size != dev.block_size()
                                    || block_size as usize > sector_buf.len()
                                {
                                    scsi.set_sense(ScsiSenseKey::IllegalRequest, 0x20, 0x00);
                                    status = CommandStatus::Failed;
                                } else {
                                    let lba = BigEndian::read_u32(&cbw.command[2..6]);
                                    let mut xferred = 0u32;
                                    let mut write_err = false;
                                    for b in 0..blocks {
                                        if self
                                            .ep_out
                                            .read(&mut sector_buf[..block_size as usize])
                                            .await
                                            .is_ok()
                                        {
                                            if dev
                                                .write_block(lba + b, &sector_buf[..block_size as usize])
                                                .is_err()
                                            {
                                                write_err = true;
                                                break;
                                            }
                                            xferred += block_size;
                                        } else {
                                            write_err = true;
                                            break;
                                        }
                                    }
                                    residue = cbw.data_transfer_length.saturating_sub(xferred);
                                    if write_err {
                                        scsi.set_sense(ScsiSenseKey::MediumError, 0x03, 0x00);
                                        status = CommandStatus::Failed;
                                    }
                                }
                            }
                            Err(st) => status = st,
                        }
                    }
                }
                _ => {
                    scsi.set_sense(ScsiSenseKey::IllegalRequest, 0x20, 0x00);
                    status = CommandStatus::Failed;
                }
            }

            // 3. Send CSW to host
            let csw = CommandStatusWrapper {
                tag: cbw.tag,
                data_residue: residue,
                status,
            };
            csw.to_bytes(&mut csw_buf);
            let _ = self.ep_in.write(&csw_buf).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scsi::RamDisk;

    #[no_mangle]
    extern "Rust" fn _embassy_time_now() -> u64 {
        0
    }

    #[no_mangle]
    extern "Rust" fn _embassy_time_schedule_wake(_at: u64, _waker: &core::task::Waker) {}

    #[test]
    fn test_ramdisk_scsi_operations() {
        let mut disk = RamDisk::<4>::new();
        let write_data = [0xAB; 512];
        assert!(disk.write_block(0, &write_data).is_ok());

        let mut read_data = [0u8; 512];
        assert!(disk.read_block(0, &mut read_data).is_ok());
        assert_eq!(write_data, read_data);

        // Out of bounds test
        assert!(disk.read_block(4, &mut read_data).is_err());
    }

    #[test]
    fn test_scsi_inquiry_handling() {
        let mut scsi = ScsiHandler::new();
        let vendor = b"TinyUSBR";
        let product = b"Mass Storage Dev";
        let revision = b"1.00";
        let mut buf = [0u8; 36];

        let len = scsi.handle_inquiry(vendor, product, revision, &mut buf);
        assert_eq!(len, 36);
        assert_eq!(&buf[8..16], vendor);
        assert_eq!(&buf[16..32], product);
        assert_eq!(&buf[32..36], revision);
        assert_eq!(scsi.sense_key, ScsiSenseKey::None);
    }

    #[test]
    fn test_scsi_request_sense_format() {
        let mut scsi = ScsiHandler::new();
        scsi.set_sense(ScsiSenseKey::NotReady, 0x3A, 0x00);
        let mut buf = [0u8; 18];
        let len = scsi.handle_request_sense(&mut buf);
        assert_eq!(len, 18);
        assert_eq!(buf[0], 0xF0);
        assert_eq!(buf[2], ScsiSenseKey::NotReady as u8);
        assert_eq!(buf[12], 0x3A);
    }

    #[test]
    fn test_scsi_read_capacity() {
        let mut scsi = ScsiHandler::new();
        let disk = RamDisk::<16>::new();
        let mut buf = [0u8; 8];

        let len = scsi.handle_read_capacity_10(&disk, &mut buf);
        assert_eq!(len, 8);
        let last_lba = BigEndian::read_u32(&buf[0..4]);
        let block_size = BigEndian::read_u32(&buf[4..8]);
        assert_eq!(last_lba, 15); // 16 blocks (0..=15)
        assert_eq!(block_size, 512);
    }
}
