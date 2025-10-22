use crate::mmio::virtio::VirtioMmio;
use crate::println;
use crate::virtio_blk;

pub fn load_bootloader(physical_base_address: usize, virtio_mmios: VirtioMmio) -> usize {
    load_virtio_blk(physical_base_address, virtio_mmios)
}

pub fn load_dtb(physical_base_address: usize, virtio_mmios: VirtioMmio) {
    load_virtio_blk(physical_base_address, virtio_mmios);
}

fn load_virtio_blk(physical_base_address: usize, virtio_mmio: VirtioMmio) -> usize {
    let mut block_device = match virtio_blk::VirtioBlk::new(virtio_mmio) {
        Ok(virtio_blk) => virtio_blk,
        Err(_) => {
            println!("can't set up block device.");
            panic!();
        }
    };

    let capacity = block_device.get_capacity();

    let mut load_address = physical_base_address;
    for i in 0..capacity {
        
        block_device.read_write_disk(load_address as *mut usize, i as u64, false);
        load_address += virtio_blk::SECTOR_SIZE;
    }

    capacity as usize * virtio_blk::SECTOR_SIZE
}
