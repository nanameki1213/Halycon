#![no_std]

extern crate alloc;

mod memory;

use alloc::vec::Vec;
use block::{BlockDevice, BlockDeviceError, SECTOR_SIZE};
use memory::allocate_pages;
use core::fmt;

const PARTITION_TYPE_FAT32: usize = 0x1c;

#[repr(C)]
pub struct PartitionTableEntry {
    bootflag: u8,
    chs_partition_start: [u8; 3],
    partition_type: u8,
    chs_partition_end: [u8; 3],
    lba_partition_start: u32,
    sector_number: u32,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(packed)]
pub struct BiosParameterBlock {
    jump_boot: [u8; 3],
    name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors_count: u16,
    fat_number: u8,
    root_entry_count: u16,
    total_sectors: u16,
    media: u8,        // unused in FAT32
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

impl BiosParameterBlock {
    pub fn new() -> Self {
        BiosParameterBlock {
            jump_boot: [0; 3],
            name: [0; 8],
            bytes_per_sector: 0,
            sectors_per_cluster: 0,
            reserved_sectors_count: 0,
            fat_number: 0,
            root_entry_count: 0,
            total_sectors: 0,
            media: 0,
            fat_size_16: 0,
            sectors_per_track: 0,
            heads_number: 0,
            hidden_sectors: 0,
            total_sectors_32: 0,
            fat_size_32: 0,
            ext_flags: 0,
            filesystem_version: 0,
            root_cluster: 0,
            filesystem_info: 0,
            bk_boot_sector: 0,
            reserved: [0; 12],
            drive_number_32: 0,
            reserved1: 0,
            boot_signature: 0,
            volume_id: 0,
            volume_label: [0; 11],
            filesystem_type: 0,
        }
    }
}

pub enum FatError {
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

pub struct Fat32<T: BlockDevice> {
    pub block_device: T,
    pub volume_start_sector: usize,
    pub bpb: BiosParameterBlock,
    pub fat: Vec<u32>,
}

impl<T: BlockDevice> Fat32<T> {
    pub fn new(mut block_device: T, volume_start_sector: usize) -> Result<Self, FatError> {
        let mut bpb_buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
        block_device.read_write_disk(
            &mut bpb_buf as *mut _ as *mut usize,
            volume_start_sector as u64,
            1,
            false,
        )?;
        let bpb = unsafe {
            let ptr = bpb_buf.as_ptr() as *const BiosParameterBlock;

            core::ptr::read_volatile(ptr)
        };

        Ok(Fat32 {
            block_device,
            volume_start_sector,
            bpb,
            fat: Vec::new(),
        })
    }

    fn get_sector(
        &mut self,
        sector_offset: usize,
        count: usize,
        buf_address: *mut usize,
    ) -> Result<(), FatError> {
        let sector = self.volume_start_sector + sector_offset;
        self.block_device
            .read_write_disk(buf_address, sector as u64, count, false)?;

        Ok(())
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        self.bpb.reserved_sectors_count as usize
            + (self.bpb.fat_number as usize * self.bpb.fat_size_32 as usize)
            + ((cluster - 2) * self.bpb.bytes_per_sector as usize)
    }

    fn get_cluster(&mut self, cluster: usize, buf_address: *mut usize) -> Result<(), FatError> {
        let sector = self.cluster_to_sector(cluster);
        self.get_sector(sector, self.bpb.sectors_per_cluster as usize, buf_address)?;

        Ok(())
    }

    fn get_next_cluster(&mut self) {}
}

const PARTITION_TABLE_OFFSET: usize = 446;
const NUM_PARTITIONS: usize = 4;

pub fn fat32_init<T>(mut block_device: T) -> Result<Fat32<T>, ()>
where
    T: BlockDevice,
{
    // read first sector to check partition table
    let mut buf: [u8; SECTOR_SIZE] = [0; SECTOR_SIZE];
    let _ = block_device.read_write_disk(buf.as_mut_ptr() as *mut usize, 0, 1, false);
    let partition_table = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            buf.as_mut_ptr().add(PARTITION_TABLE_OFFSET) as *mut PartitionTableEntry,
            NUM_PARTITIONS,
        )
    };

    for partition in partition_table {
        if partition.partition_type as usize == PARTITION_TYPE_FAT32 {
            match Fat32::new(block_device, partition.lba_partition_start as usize) {
                Ok(fat32) => return Ok(fat32),
                Err(_) => return Err(()),
            }
        }
    }

    return Err(());
}
