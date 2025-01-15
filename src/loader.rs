use crate::allocate_memory;
use crate::virtio;
use crate::virtio_blk;
use crate::paging;

pub fn load_bootloader(physical_base_address: usize) -> usize {
    load_virtio_blk(physical_base_address)
}

pub fn load_dtb(physical_base_address: usize) {
    unsafe {
        virtio::VIRTIO_MMIO_ADDRESS = 0x10002000;
    }
    load_virtio_blk(physical_base_address);
}

fn load_virtio_blk(physical_base_address: usize) -> usize{
    let capacity = unsafe {
        core::ptr::read_volatile((virtio::VIRTIO_MMIO_ADDRESS + 0x100) as *mut u64)
    };
    
    virtio_blk::init_virtio_blk();
    let vq = virtio::init_virtio_mmio(virtio::VIRTIO_DEFAULT_INDEX).unwrap();

    unsafe {
        let virtio_blk_req = &mut *(allocate_memory(1, paging::PAGE_SIZE).unwrap() as *mut virtio_blk::VirtioBlkReq);
        let mut load_address = physical_base_address;
        for i in 0..capacity {
            virtio_blk::read_write_disk(&mut *vq, load_address as *mut usize, virtio_blk_req, i, false);
            load_address += virtio_blk::SECTOR_SIZE;
        }
    }

    capacity as usize * virtio_blk::SECTOR_SIZE
}
