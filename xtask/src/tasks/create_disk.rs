use fatfs::{self, FileSystem, FormatVolumeOptions, FsOptions};
use mbrman;
use std::{
    io::{Cursor, Seek},
    path::Path,
};

type DynError = Box<dyn std::error::Error>;

const SECTOR_SIZE: usize = 512;
const PARTITION_TYPE_FAT32: u8 = 0x1c;

pub fn create_fat32_disk(image_path: &Path, files_path: &[&Path]) -> Result<(), DynError> {
    let disk_size = 1024 * 1024 * 128; // 128MiB
    let num_sectors = disk_size / SECTOR_SIZE;

    let partition_start_lba = 2048;
    let partition_sectors = num_sectors - partition_start_lba;

    let mut data = vec![0; disk_size];
    let mut cur = Cursor::new(&mut data);

    let mut mbr = mbrman::MBR::new_from(&mut cur, num_sectors as u32, [0xff; 4])
        .expect("could not create partition table");

    mbr[1] = mbrman::MBRPartitionEntry {
        boot: mbrman::BOOT_ACTIVE,
        first_chs: mbrman::CHS::empty(),
        sys: PARTITION_TYPE_FAT32,
        last_chs: mbrman::CHS::empty(),
        starting_lba: partition_start_lba as u32,
        sectors: partition_sectors as u32,
    };
    mbr.write_into(&mut cur)?;
    let partition_start_byte = partition_start_lba as usize * SECTOR_SIZE;
    let partition_num_bytes = partition_sectors as usize * SECTOR_SIZE;
    let mbr_partition_range = partition_start_byte..partition_start_byte + partition_num_bytes;

    init_fat(&mut data[mbr_partition_range], files_path)?;

    fs_err::write(image_path, &data)?;
    log::info!("Wrote disk image to: {}", image_path.display());

    Ok(())
}

fn init_fat(partition: &mut [u8], files_path: &[&Path]) -> Result<(), DynError> {
    let fat32_fs = {
        let mut cur = Cursor::new(partition);
        fatfs::format_volume(
            &mut cur,
            FormatVolumeOptions::new().fat_type(fatfs::FatType::Fat32),
        )?;

        cur.rewind()?;
        FileSystem::new(cur, FsOptions::new().update_accessed_date(false))?
    };

    let root_dir = fat32_fs.root_dir();
    for src_file in files_path {
        let mut file = root_dir.create_file(src_file.file_name().unwrap().to_str().unwrap())?;
        log::info!(
            "create file: {}",
            src_file.file_name().unwrap().to_str().unwrap()
        );
        std::io::copy(&mut fs_err::File::open(src_file)?, &mut file)?;
    }

    log::info!("{:?}", fat32_fs.stats()?);

    Ok(())
}
