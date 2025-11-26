use crate::println;
use block::BlockDevice;
use block::virtio_blk;
use virtio::VirtioMmio;

pub fn load_bootloader(physical_base_address: usize, virtio_mmios: VirtioMmio) -> usize {
    load_virtio_blk(physical_base_address, virtio_mmios)
}

pub fn load_dtb(physical_base_address: usize, virtio_mmios: VirtioMmio) {
    load_virtio_blk(physical_base_address, virtio_mmios);
}

fn load_virtio_blk(physical_base_address: usize, virtio_mmio: VirtioMmio) -> usize {
    let mut block_device = match virtio_blk::VirtioBlk::new(virtio_mmio) {
        Ok(virtio_blk) => virtio_blk,
        Err(err) => {
            println!("can't set up block device: {}", err);
            panic!();
        }
    };

    let capacity = block_device.get_capacity();

    let load_address = physical_base_address;
    let _ = block_device.read_write_disk(load_address as *mut usize, 0, capacity, false);
    // for i in 0..capacity {
    //     block_device.read_write_disk(load_address as *mut usize, i as u64, false);
    //     load_address += virtio_blk::SECTOR_SIZE;
    // }

    capacity as usize * block::SECTOR_SIZE
}
