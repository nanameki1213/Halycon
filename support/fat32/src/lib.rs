#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::{string::String, vec};
use block::{BlockDevice, BlockDeviceError, SECTOR_SIZE};
use core::fmt;
use core::str::FromStr;

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

#[derive(Debug)]
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

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct DirEntry {
    name: [u8; 8],
    ext: [u8; 3],
    attributes: u8,
    reserved: u8,
    creation_tenths: u8,
    creation_time: u16,
    creation_date: u16,
    last_access_date: u16,
    first_cluster_high: u16,
    last_write_time: u16,
    last_write_data: u16,
    first_cluster_low: u16,
    file_size: u32,
}

const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

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

        let fat_start_sector = bpb.reserved_sectors_count as usize;
        let fat_bytes = bpb.fat_size_32 as usize * bpb.bytes_per_sector as usize;

        let fat_num_of_sectors = if fat_bytes % SECTOR_SIZE == 0 {
            fat_bytes / SECTOR_SIZE
        } else {
            fat_bytes / SECTOR_SIZE + 1
        };

        let mut fat = vec![0u32; fat_bytes / core::mem::size_of::<u32>()];

        block_device.read_write_disk(
            fat.as_mut_ptr() as *mut usize,
            fat_start_sector as u64,
            fat_num_of_sectors,
            false,
        )?;

        Ok(Fat32 {
            block_device,
            volume_start_sector,
            bpb,
            fat,
        })
    }

    fn get_sector(
        &mut self,
        sector_offset: usize,
        count: usize,
        buf: &mut [u8],
    ) -> Result<(), FatError> {
        let sector = self.volume_start_sector + sector_offset;
        let buf_ptr = buf.as_mut_ptr() as *mut usize;

        self.block_device
            .read_write_disk(buf_ptr, sector as u64, count, false)?;

        Ok(())
    }

    fn cluster_to_sector(&self, cluster: usize) -> usize {
        self.bpb.reserved_sectors_count as usize
            + (self.bpb.fat_number as usize * self.bpb.fat_size_32 as usize)
            + ((cluster - 2) * self.bpb.sectors_per_cluster as usize)
    }

    fn get_cluster(&mut self, cluster: usize, buf: &mut [u8]) -> Result<(), FatError> {
        let sector = self.cluster_to_sector(cluster);
        self.get_sector(sector, self.bpb.sectors_per_cluster as usize, buf)?;

        Ok(())
    }

    fn get_next_cluster(&mut self, cluster: usize) -> usize {
        self.fat[cluster] as usize
    }

    fn list_root_files(&mut self) -> Result<Vec<String>, FatError> {
        let mut file_list = Vec::new();

        const PAGE_SIZE: usize = 0x1000;

        // get root directory cluster number
        let bytes_per_cluster =
            (self.bpb.sectors_per_cluster as u16 * self.bpb.bytes_per_sector) as usize;
        let mut buf = vec![0u8; bytes_per_cluster];
        self.get_cluster(self.bpb.root_cluster as usize, &mut buf)?;

        let (_, dir, _) = unsafe { buf.align_to_mut::<DirEntry>() };

        for entry in dir {
            if entry.attributes != ATTR_LONG_NAME {
                // TODO: if file name is broken, should reference copy of fat.
                let name = str::from_utf8(&entry.name).expect("file name is broken.");
                file_list.push(String::from_str(name).unwrap());
            }
        }

        Ok(file_list)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::PathBuf;
    use std::path::Path;

    struct StdFileBlockDevice {
        file: File,
    }

    impl StdFileBlockDevice {
        fn new(image_path: PathBuf) -> Self {
            let file = File::open(&image_path)
                .expect("Failed to open disk image");

            Self { file }
        }
    }

    impl BlockDevice for StdFileBlockDevice {
        fn read_write_disk(
                &mut self,
                buf_address: *mut usize,
                sector: u64,
                count: usize,
                is_write: bool,
            ) -> Result<(), BlockDeviceError> {
            if is_write {
                return Ok(());
            }

            Ok(())
        }

        fn get_capacity(&self) -> usize {
            0
        }
    }

    #[test]
    fn test_cluster_to_sector() {
        let mut image_path = Path::new(&env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .unwrap()
                .to_path_buf();
        image_path.push("test_disk.img");

        let device = StdFileBlockDevice::new(image_path);
        let fs = Fat32::new(device, 0).expect("Failed to parse BPB from image");

        // データ領域の開始セクタ = Reserved + (FAT数 * FATサイズ)
        let data_start_sector = fs.bpb.reserved_sectors_count as usize
            + (fs.bpb.fat_number as usize * fs.bpb.fat_size_32 as usize);

        let cluster_2_sector = fs.cluster_to_sector(2);
        assert_eq!(
            cluster_2_sector,
            data_start_sector,
            "Cluster 2 should match data start sector"
        );

        // クラスタ3 の位置検証
        let cluster_3_sector = fs.cluster_to_sector(3);
        let expected_cluster_3 = data_start_sector + fs.bpb.sectors_per_cluster as usize;
        assert_eq!(
            cluster_3_sector,
            expected_cluster_3,
            "Cluster 3 should be 1 cluster size away from Cluster 2"
        );
    }
}
