#![no_std]

use block::{
    BlockDevice,
    BlockDeviceError,
    SECTOR_SIZE,
};
use core::fmt;

#[repr(C)]
pub struct PartitionTableEntry {
    bootflag: u8,
    chs_partition_start: [u8; 3],
    partition_type: u8,
    chs_partition_end: [u8; 3],
    lba_partition_start: u32,
    sector_number: u32,
}

#[repr(C)]
pub struct BiosParameterBlock {
    jump_boot: [u8; 3],
    name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors_count: u16,
    fat_number: u8,
    root_entry_count: u16,
    total_sectors: u16,
    media: u8, // unused in FAT32
    fat_size_16: u16, // unused in FAT32
    sectors_per_track: u16,
    heads_number: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    fat_size_32: u32,
    ext_flags: u16,
    filesystem_version: u16,
    root_cluster: u32,
    filesystem_info: u16,
    bk_boot_sector: u16,
    reserved: [u8; 12],
    drive_number_32: u8,
    reserved1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    filesystem_type: u64,
}

enum FatError {
    DeviceError(BlockDeviceError),
}

impl fmt::Display for FatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceError(err) => write!(f, "block device error: {}", err),
        }
    }
}

impl From<BlockDeviceError> for FatError {
    fn from(value: BlockDeviceError) -> Self {
        FatError::DeviceError(value)
    }
}

const PARTITION_TABLE_OFFSET: usize = 446;
const NUM_PARTITIONS: usize = 4;

pub fn fat32_init<T>(block_device: T) -> Result<(), FatError>
where
    T: BlockDevice
{
    // read first sector to check partition table
    let buf: [u8; SECTOR_SIZE];
    block_device.read_write_disk(buf.as_mut_ptr() as *mut usize, 0, 1, false)?;
    let partition_table = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(buf.as_mut_ptr().add(PARTITION_TABLE_OFFSET) as *mut PartitionTableEntry, NUM_PARTITIONS)
    };
}
