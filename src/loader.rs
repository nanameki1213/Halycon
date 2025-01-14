use crate::allocate_memory;
use crate::virtio::*;
use crate::virtio_blk::*;
use crate::PAGE_SHIFT;

// return entry_point
pub fn load_bootloader() -> usize {
    load_virtio_blk()
}

fn load_virtio_blk() -> usize {
    let capacity = unsafe {
        core::ptr::read_volatile((VIRTIO_MMIO_ADDRESS + 0x100) as *mut u64)
    };
    
    init_virtio_blk();
    let vq = init_virtio_mmio(VIRTIO_DEFAULT_INDEX).unwrap();

    unsafe {
        let virtio_blk_req = &mut *(allocate_memory(1, 1 << PAGE_SHIFT).unwrap() as *mut VirtioBlkReq);
        let entry_point = allocate_memory(capacity as usize * 8, 1 << PAGE_SHIFT).unwrap();
        let mut load_address = entry_point;
        for i in 0..capacity {
            read_write_disk(&mut *vq, load_address as *mut usize, virtio_blk_req, i, false);
            load_address += SECTOR_SIZE;
        }

        entry_point
    }
}
