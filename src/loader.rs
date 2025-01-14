use crate::allocate_memory;
use crate::virtio::*;
use crate::virtio_blk::*;
use crate::PAGE_SHIFT;

pub fn load_bootloader() {
    load_virtio_blk();
}

fn load_virtio_blk() {
    let capacity = unsafe {
        core::ptr::read_volatile((VIRTIO_MMIO_ADDRESS + 0x100) as *mut u64)
    };
    
    init_virtio_blk();
    let vq = init_virtio_mmio(VIRTIO_DEFAULT_INDEX).unwrap();

    unsafe {
        let virtio_blk_req = &mut *(allocate_memory(1, 1 << PAGE_SHIFT).unwrap() as *mut VirtioBlkReq);
        let mut load_address = allocate_memory(capacity as usize * 8, 1 << PAGE_SHIFT).unwrap();
        for i in 0..capacity {
            read_write_disk(&mut *vq, load_address as *mut usize, virtio_blk_req, i, false);
            load_address += SECTOR_SIZE;
        }
    }
}
