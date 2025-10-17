extern crate alloc;

use crate::mmio::virtio;
use crate::virtio_blk;
use crate::virtio_blk::VirtioBlkReq;
use alloc::boxed::Box;

pub fn load_bootloader(physical_base_address: usize) -> usize {
    load_virtio_blk(physical_base_address)
}

pub fn load_dtb(physical_base_address: usize) {
    unsafe {
        virtio::VIRTIO_MMIO_ADDRESS = 0x10002000;
    }
    load_virtio_blk(physical_base_address);
}

fn load_virtio_blk(physical_base_address: usize) -> usize {
    let capacity =
        unsafe { core::ptr::read_volatile((virtio::VIRTIO_MMIO_ADDRESS + 0x100) as *mut u64) };

    virtio_blk::init_virtio_blk();
    let vq = virtio::init_virtio_mmio(virtio::VIRTIO_DEFAULT_INDEX).unwrap();

    unsafe {
        // TODO: VirtioBlkReqはnewメソッドを実装する
        let mut virtio_blk_req: Box<VirtioBlkReq> = Box::new(VirtioBlkReq {
            req_type: 0,
            reserved: 0,
            sector: 0,
            data: [0; 512],
            status: 0,
        });
        let mut load_address = physical_base_address;
        for i in 0..capacity {
            virtio_blk::read_write_disk(
                &mut *vq,
                load_address as *mut usize,
                virtio_blk_req.as_mut(),
                i,
                false,
            );
            load_address += virtio_blk::SECTOR_SIZE;
        }
    }

    capacity as usize * virtio_blk::SECTOR_SIZE
}
