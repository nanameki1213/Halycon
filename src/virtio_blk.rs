#![allow(dead_code)]

use crate::println;
use crate::virtio::*;
use core::mem::size_of;
use core::ptr::slice_from_raw_parts_mut;
use core::usize;

pub const SECTOR_SIZE: usize = 512;

pub const VIRTIO_BLK_T_IN: usize = 0;
pub const VIRTIO_BLK_T_OUT: usize = 1;
pub const VIRTIO_BLK_T_FLUSH: usize = 4;
pub const VIRTIO_BLK_T_DISCARD: usize = 11;
pub const VIRTIO_BLK_T_WRITE_ZEROES: usize = 13;

pub const VIRTIO_BLK_S_OK: usize = 0;
pub const VIRTIO_BLK_S_IOERR: usize = 1;
pub const VIRTIO_BLK_S_UNSUPP: usize = 2;

#[repr(C)]
pub struct VirtioBlkReq {
    pub req_type: u32,
    pub reserved: u32,
    pub sector: u64,
    pub data: [u8; 512],
    pub status: u8,
}

pub fn init_virtio_blk() {
    // 1. Reset the device.
    set_virtio_mmio(VIRTIO_MMIO_STATUS, 0x0);
    // 2. Set the ACKNOWLEDGE status bit
    set_virtio_mmio(VIRTIO_MMIO_STATUS, VIRTIO_MMIO_STATUS_ACKNOWLEDGE as u32);
    // 3. Set the DRIVER status bit
    let mut status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_DRIVER as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
    // 4. Read device feature bits, and write the subset of feature bits understood
    //    by the OS and driver to the device.
    // TODO: setting feature bits

    // 5. Set the FEATURES_OK status bit.
    status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
    // 6. Re-read device status to ensure the FEATURES_OK bit is still set
    // TODO: setting feature bits

    // 7. Perform device-specific setup

    // 8. Set the DRIVER_OK status bit. At this point the device is "live".
    status = get_virtio_mmio(VIRTIO_MMIO_STATUS);
    status |= VIRTIO_MMIO_STATUS_DRIVER_OK as u32;
    set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
}

pub fn read_write_disk(
    queue: &mut VirtQueue,
    buf_address: *mut usize,
    virtio_blk_req: &mut VirtioBlkReq,
    sector: u64,
    is_write: bool,
) {
    // make a request
    virtio_blk_req.sector = sector;
    virtio_blk_req.status = 0xff;
    if is_write {
        let bytes = unsafe { &mut *slice_from_raw_parts_mut(buf_address as *mut u8, SECTOR_SIZE) };
        virtio_blk_req.data[..SECTOR_SIZE].copy_from_slice(&bytes);
    }

    // setting Virtqueue
    let desc = &mut queue.vring.desc;
    desc[0].addr = virtio_blk_req as *const VirtioBlkReq as u64;
    desc[0].len = size_of::<u32>() as u32 * 2 + size_of::<u64>() as u32;
    desc[0].flags = VRingDesc::VIRTQ_DESC_F_NEXT as u16;
    desc[0].next = 1;

    desc[1].addr = virtio_blk_req as *const VirtioBlkReq as u64 + desc[0].len as u64;
    desc[1].len = SECTOR_SIZE as u32;
    desc[1].flags = VRingDesc::VIRTQ_DESC_F_NEXT as u16;
    if !is_write {
        desc[1].flags |= VRingDesc::VIRTQ_DESC_F_WRITE as u16;
    }
    desc[1].next = 2;

    // status field in VirtioBlkReq
    desc[2].addr =
        virtio_blk_req as *const VirtioBlkReq as u64 + (desc[0].len + desc[1].len) as u64;
    desc[2].len = size_of::<u8>() as u32;
    desc[2].flags = VRingDesc::VIRTQ_DESC_F_WRITE as u16;
    desc[2].next = 0;

    connect_to_avail_ring(queue, 0);
    queue.last_used_index += 1;

    notify_to_device(VIRTIO_DEFAULT_INDEX);

    while queue.last_used_index != unsafe { core::ptr::read_volatile(&queue.vring.used.idx) } {}

    if virtio_blk_req.status != VIRTIO_BLK_S_OK as u8 {
        println!(
            "Virtio-Blk Error: {}",
            match virtio_blk_req.status as usize {
                VIRTIO_BLK_S_IOERR => "Input/Output ERROR",
                VIRTIO_BLK_S_UNSUPP => "UNSUPPORTED BY DEVICE",
                _ => unreachable!(),
            }
        );
        panic!();
    }

    if !is_write {
        let bytes = unsafe { &mut *slice_from_raw_parts_mut(buf_address as *mut u8, SECTOR_SIZE) };
        bytes.copy_from_slice(&virtio_blk_req.data[..SECTOR_SIZE]);
    }
}
