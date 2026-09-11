//! SCSI command processing and BlockDevice trait.
use byteorder::{BigEndian, ByteOrder};
use tinyusb_types::class::msc::ScsiSenseKey;

pub trait BlockDevice {
    fn block_count(&self) -> u32;
    fn block_size(&self) -> u32 {
        512
    }
    fn is_writable(&self) -> bool {
        true
    }
    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), ()>;
    fn write_block(&mut self, lba: u32, buf: &[u8]) -> Result<(), ()>;
}

/// In-memory RAM disk implementation of BlockDevice for testing and fast prototyping.
pub struct RamDisk<const BLOCKS: usize> {
    data: [[u8; 512]; BLOCKS],
}

impl<const BLOCKS: usize> RamDisk<BLOCKS> {
    pub const fn new() -> Self {
        Self {
            data: [[0u8; 512]; BLOCKS],
        }
    }
}

impl<const BLOCKS: usize> Default for RamDisk<BLOCKS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const BLOCKS: usize> BlockDevice for RamDisk<BLOCKS> {
    fn block_count(&self) -> u32 {
        BLOCKS as u32
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), ()> {
        if (lba as usize) < BLOCKS && buf.len() >= 512 {
            buf[..512].copy_from_slice(&self.data[lba as usize]);
            Ok(())
        } else {
            Err(())
        }
    }

    fn write_block(&mut self, lba: u32, buf: &[u8]) -> Result<(), ()> {
        if (lba as usize) < BLOCKS && buf.len() >= 512 {
            self.data[lba as usize].copy_from_slice(&buf[..512]);
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct ScsiHandler {
    pub sense_key: ScsiSenseKey,
    pub asc: u8,
    pub ascq: u8,
}

impl Default for ScsiHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ScsiHandler {
    pub const fn new() -> Self {
        Self {
            sense_key: ScsiSenseKey::None,
            asc: 0,
            ascq: 0,
        }
    }

    pub fn set_sense(&mut self, key: ScsiSenseKey, asc: u8, ascq: u8) {
        self.sense_key = key;
        self.asc = asc;
        self.ascq = ascq;
    }

    pub fn handle_inquiry(
        &mut self,
        vendor: &[u8; 8],
        product: &[u8; 16],
        revision: &[u8; 4],
        buf: &mut [u8],
    ) -> usize {
        let resp_len = 36.min(buf.len());
        buf[..resp_len].fill(0);

        buf[0] = 0x00; // Direct access block device
        buf[1] = 0x80; // Removable media
        buf[2] = 0x02; // ANSI SCSI-2
        buf[3] = 0x02; // Response data format
        buf[4] = 31;   // Additional length

        if resp_len >= 16 {
            buf[8..16].copy_from_slice(vendor);
        }
        if resp_len >= 32 {
            buf[16..32].copy_from_slice(product);
        }
        if resp_len >= 36 {
            buf[32..36].copy_from_slice(revision);
        }

        self.set_sense(ScsiSenseKey::None, 0, 0);
        resp_len
    }

    pub fn handle_read_capacity_10<B: BlockDevice>(&mut self, dev: &B, buf: &mut [u8]) -> usize {
        if buf.len() < 8 {
            self.set_sense(ScsiSenseKey::IllegalRequest, 0x24, 0x00);
            return 0;
        }

        let total_blocks = dev.block_count();
        let block_size = dev.block_size();
        if total_blocks == 0 || block_size == 0 {
            self.set_sense(ScsiSenseKey::NotReady, 0x3A, 0x00);
            return 0;
        }

        let last_lba = total_blocks - 1;

        BigEndian::write_u32(&mut buf[0..4], last_lba);
        BigEndian::write_u32(&mut buf[4..8], block_size);

        self.set_sense(ScsiSenseKey::None, 0, 0);
        8
    }

    pub fn handle_request_sense(&mut self, buf: &mut [u8]) -> usize {
        let len = 18.min(buf.len());
        buf[..len].fill(0);

        // Fixed-format sense: response code 0x70 with Valid bit set (see `scsi_sense_fixed_resp_t`).
        buf[0] = 0xF0;
        buf[2] = self.sense_key as u8 & 0x0F;
        buf[7] = 10; // Additional sense length (sizeof fixed sense - 8)
        buf[12] = self.asc;
        buf[13] = self.ascq;

        self.set_sense(ScsiSenseKey::None, 0, 0);
        len
    }

    pub fn handle_mode_sense_6(&mut self, writable: bool, buf: &mut [u8]) -> usize {
        if buf.len() < 4 {
            return 0;
        }
        buf[0] = 3; // Mode data length
        buf[1] = 0; // Medium type
        buf[2] = if writable { 0 } else { 0x80 }; // Write-protect bit
        buf[3] = 0; // Block descriptor length
        self.set_sense(ScsiSenseKey::None, 0, 0);
        4
    }
}
