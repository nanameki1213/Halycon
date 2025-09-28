use crate::mmio::virtio::VIRTIO_MMIO_DEFAULT_ADDRESS;
use crate:: virtio_blk;
use crate::println;

pub fn load_bootloader(physical_base_address: usize) -> usize {
    // TODO: 0x10001000がマジックバリュー
    load_virtio_blk(physical_base_address, 0x10001000)
}

pub fn load_dtb(physical_base_address: usize) {
    // TODO: 0x10002000がマジックバリュー
    load_virtio_blk(physical_base_address, 0x10002000);
}

fn load_virtio_blk(physical_base_address: usize, mmio_base_address: usize) -> usize {

    let mut block_device = match virtio_blk::VirtioBlk::new(mmio_base_address) {
        Ok(virtio_blk) => virtio_blk,
        Err(_) => {
            println!("can't set up block device.");
            panic!();
        }
    };

    let capacity = block_device.get_capacity();
    block_device.init_virtio_blk();

    // TODO: VirtioBlkReqはnewメソッドを実装する
   let mut load_address = physical_base_address;
    for i in 0..capacity {
        block_device.read_write_disk(
            load_address as *mut usize,
            i as u64,
            false,
        );
        load_address += virtio_blk::SECTOR_SIZE;
    }

    capacity as usize * virtio_blk::SECTOR_SIZE
}
