#![cfg_attr(not(test), no_std)]
#![feature(assert_matches)]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{string::String, vec};
use block::{BlockDevice, BlockDeviceError, SECTOR_SIZE};
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

#[derive(Clone, Copy, Debug)]
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
    NoSuchFileOrDirectory,
    OutOfBuffer,
}

impl fmt::Display for FatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceError(err) => write!(f, "block device error: {}", err),
            Self::NoSuchFileOrDirectory => write!(f, "No such file or directory."),
            Self::OutOfBuffer => write!(f, "Buffer too small."),
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
#[allow(dead_code)]
const ATTR_DIRECTORY: u8 = 0x10;
#[allow(dead_code)]
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

        let fat_start_sector = volume_start_sector + bpb.reserved_sectors_count as usize;
        let fat_bytes = bpb.fat_size_32 as usize * bpb.bytes_per_sector as usize;

        let mut fat = vec![0u32; fat_bytes / core::mem::size_of::<u32>()];

        block_device.read_write_disk(
            fat.as_mut_ptr() as *mut usize,
            fat_start_sector as u64,
            bpb.fat_size_32 as usize,
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

    fn cluster_to_sector(&self, cluster: u32) -> usize {
        self.bpb.reserved_sectors_count as usize
            + (self.bpb.fat_number as usize * self.bpb.fat_size_32 as usize)
            + ((cluster as usize - 2) * self.bpb.sectors_per_cluster as usize)
    }

    fn get_cluster(&mut self, cluster: u32, buf: &mut [u8]) -> Result<(), FatError> {
        let sector = self.cluster_to_sector(cluster);
        self.get_sector(sector, self.bpb.sectors_per_cluster as usize, buf)?;

        Ok(())
    }

    fn get_next_cluster(&mut self, cluster: u32) -> u32 {
        self.fat[cluster as usize]
    }

    fn get_directory_entries(
        &mut self,
        dir_cluster_number: u32,
    ) -> Result<Vec<DirEntry>, FatError> {
        // get root directory cluster number
        let bytes_per_cluster =
            (self.bpb.sectors_per_cluster as u16 * self.bpb.bytes_per_sector) as usize;
        let mut buf = vec![0u8; bytes_per_cluster];
        self.get_cluster(dir_cluster_number, &mut buf)?;

        let entry_size = core::mem::size_of::<DirEntry>();
        let num_entries = buf.len();

        let mut dir_entries = Vec::with_capacity(num_entries);
        for i in 0..num_entries {
            unsafe {
                let ptr = buf.as_ptr().add(i * entry_size) as *const DirEntry;
                dir_entries.push(ptr.read_unaligned());
            }
        }

        Ok(dir_entries)
    }

    fn get_short_file_name(&self, name: &[u8; 8], ext: &[u8; 3]) -> String {
        // file name
        let name_str = String::from_utf8_lossy(name).trim_end().to_string();

        // file extension
        let ext_str = String::from_utf8_lossy(ext).trim_end().to_string();

        if !ext_str.is_empty() {
            let file_name = [name_str, ext_str].join(".");
            return file_name;
        }

        return name_str;
    }

    pub fn list_root_files(&mut self) -> Result<Vec<String>, FatError> {
        let mut file_list = Vec::new();

        let dir_entries = self.get_directory_entries(self.bpb.root_cluster)?;

        for entry in dir_entries {
            if entry.name[0] == 0x0 {
                break;
            }
            if entry.attributes != ATTR_LONG_NAME {
                let name = self.get_short_file_name(&entry.name, &entry.ext);
                file_list.push(name);
            }
        }

        Ok(file_list)
    }

    fn get_directory_entry(
        &mut self,
        dir_cluster_number: u32,
        name: &String,
    ) -> Result<DirEntry, FatError> {
        let dir_entries = self.get_directory_entries(dir_cluster_number)?;

        for entry in dir_entries {
            let file_name = self.get_short_file_name(&entry.name, &entry.ext);
            if file_name == *name {
                return Ok(entry);
            }
        }

        return Err(FatError::NoSuchFileOrDirectory);
    }

    pub fn get_file_size(&mut self, name: &String) -> Result<usize, FatError> {
        let entry = self.get_directory_entry(self.bpb.root_cluster, name)?;
        Ok(entry.file_size as usize)
    }

    pub fn read_file(&mut self, name: &String, buf: &mut [u8]) -> Result<usize, FatError> {
        let entry = self.get_directory_entry(self.bpb.root_cluster, name)?;
        if buf.len() < entry.file_size as usize {
            return Err(FatError::OutOfBuffer);
        }
        let mut buf_address = buf.as_mut_ptr();
        let mut current_cluster_number =
            (entry.first_cluster_high as u32) << 16 | entry.first_cluster_low as u32;
        let bytes_per_cluster =
            (self.bpb.sectors_per_cluster as u16 * self.bpb.bytes_per_sector) as usize;

        while current_cluster_number < 0x0FFFFFF8 {
            unsafe {
                let buf = core::slice::from_raw_parts_mut(buf_address, bytes_per_cluster);

                self.get_cluster(current_cluster_number, buf)?;

                current_cluster_number = self.get_next_cluster(current_cluster_number);
                buf_address = buf_address.add(bytes_per_cluster);
            }
        }

        Ok(entry.file_size as usize)
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
    use std::assert_matches::assert_matches;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};
    use std::path::Path;
    use std::path::PathBuf;

    struct StdFileBlockDevice {
        file: File,
    }

    impl StdFileBlockDevice {
        fn new(image_path: PathBuf) -> Self {
            let file = File::open(&image_path).expect("Failed to open disk image");

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

            let offset = sector * SECTOR_SIZE as u64;

            self.file
                .seek(SeekFrom::Start(offset))
                .map_err(|_| BlockDeviceError::IOError)?;

            let buf_u8 = unsafe {
                std::slice::from_raw_parts_mut(buf_address as *mut u8, count * SECTOR_SIZE)
            };

            self.file
                .read_exact(buf_u8)
                .map_err(|_| BlockDeviceError::IOError)?;

            Ok(())
        }

        fn get_capacity(&self) -> usize {
            0
        }
    }

    fn get_volume() -> Fat32<StdFileBlockDevice> {
        let mut image_path = Path::new(&env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .unwrap()
            .to_path_buf();
        image_path.push("test_disk.img");

        let device = StdFileBlockDevice::new(image_path);
        let fs = fat32_init(device).expect("Failed to init fat32");

        fs
    }

    #[test]
    fn test_cluster_to_sector() {
        let fs = get_volume();

        // データ領域の開始セクタ = Reserved + (FAT数 * FATサイズ)
        let data_start_sector = fs.bpb.reserved_sectors_count as usize
            + (fs.bpb.fat_number as usize * fs.bpb.fat_size_32 as usize);

        let cluster_2_sector = fs.cluster_to_sector(2);
        assert_eq!(
            cluster_2_sector, data_start_sector,
            "Cluster 2 should match data start sector"
        );

        // クラスタ3 の位置検証
        let cluster_3_sector = fs.cluster_to_sector(3);
        let expected_cluster_3 = data_start_sector + fs.bpb.sectors_per_cluster as usize;
        assert_eq!(
            cluster_3_sector, expected_cluster_3,
            "Cluster 3 should be 1 cluster size away from Cluster 2"
        );
    }

    #[test]
    fn test_list_root_files() {
        let mut fs = get_volume();

        let files = fs.list_root_files().expect("Failed to get file list");

        assert_eq!(files[0], "TEST1.TXT");
        assert_eq!(files[1], "TEST2.TXT");
        assert_eq!(files[2], "TEST3.TXT");
        assert_eq!(files[3], "TEST4.TXT");
    }

    #[test]
    fn test_read_file() {
        let mut fs = get_volume();

        println!("bpb: {:?}", fs.bpb);
        println!("fs: {}", fs.volume_start_sector);

        let files = fs.list_root_files().expect("Failed to get file list");

        let mut buf = [0u8; SECTOR_SIZE * 8];
        fs.read_file(&files[0], &mut buf)
            .expect("Failed to read file");

        let test_str = "The process of analyzing a FAT32 file system using a hex editor requires a deep understanding of how data is structured across sectors and clusters. This specific paragraph is designed to exceed the standard sector size of 512 bytes, ensuring that your read test can verify whether the file system driver or your manual parsing logic correctly handles data that spans across multiple sectors. When you examine this file in a hex dump, you should notice that the text continues past the first 0x200 bytes offset. If the file is stored in cluster 2, for example, you can calculate its physical location by identifying the start of the data region. Remember that in FAT32, the root directory is no longer at a fixed location but is treated as a cluster chain. This provides more flexibility compared to older FAT versions. By reading this entire passage successfully, you confirm that your environment can handle basic file I/O operations and that your offset calculations from the MBR to the BPB, and finally to the data area, are accurate.".as_bytes();
        let mut test_buf = [0u8; 4096];
        let len = test_str.len().min(4096);
        test_buf[..len].copy_from_slice(&test_str[..len]);
        assert_eq!(buf, test_buf);
    }

    #[test]
    fn test_out_of_buffer() {
        let mut fs = get_volume();

        let files = fs.list_root_files().expect("Failed to get file list");

        let size = fs
            .get_file_size(&files[0])
            .expect("Failed to get file size");
        let mut buf = vec![0u8; size - 1];
        assert_matches!(
            fs.read_file(&files[0], &mut buf),
            Err(FatError::OutOfBuffer)
        )
    }
}
