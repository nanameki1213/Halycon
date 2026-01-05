extern crate alloc;

use crate::BlockDevice;
use crate::BlockDeviceError;
use crate::SECTOR_SIZE;
use alloc::vec::Vec;
use core::slice;

#[derive(Debug)]
pub struct MemBlk {
    pub data: Vec<u8>,
}

impl MemBlk {
    pub fn new(data: &[u8]) -> Self {
        MemBlk {
            data: data.to_vec(),
        }
    }
}

impl BlockDevice for MemBlk {
    fn read_write_disk(
        &mut self,
        buf_address: *mut usize,
        sector: u64,
        count: usize,
        is_write: bool,
    ) -> Result<(), BlockDeviceError> {
        let start_offset = sector as usize * SECTOR_SIZE;
        let total_bytes = count * SECTOR_SIZE;
        let end_offset = start_offset + total_bytes;

        if end_offset > self.data.len() {
            return Err(BlockDeviceError::IOError);
        }

        unsafe {
            // TODO: buf_address type should be *mut u8
            let buf_ptr = buf_address as *mut u8;

            let buf_slice = slice::from_raw_parts_mut(buf_ptr, total_bytes);
            let disk_slice = &mut self.data[start_offset..end_offset];

            if is_write {
                disk_slice.copy_from_slice(buf_slice);
            } else {
                buf_slice.copy_from_slice(disk_slice);
            }
        }

        Ok(())
    }

    fn get_capacity(&self) -> usize {
        self.data.len() / SECTOR_SIZE
    }
}
