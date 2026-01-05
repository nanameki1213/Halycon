#![allow(dead_code)]

extern crate alloc;

use alloc::boxed::Box;
use core::mem::size_of;
use core::usize;
use virtio::*;

use crate::BlockDevice;
use crate::BlockDeviceError;
use crate::SECTOR_SIZE;

pub const VIRTIO_BLK_T_IN: usize = 0;
pub const VIRTIO_BLK_T_OUT: usize = 1;
pub const VIRTIO_BLK_T_FLUSH: usize = 4;
pub const VIRTIO_BLK_T_DISCARD: usize = 11;
pub const VIRTIO_BLK_T_WRITE_ZEROES: usize = 13;

pub const VIRTIO_BLK_S_OK: usize = 0;
pub const VIRTIO_BLK_S_IOERR: usize = 1;
pub const VIRTIO_BLK_S_UNSUPP: usize = 2;

// block device feature bits
pub const VIRTIO_BLK_F_SIZE_MAX: u64 = 1 << 1;
pub const VIRTIO_BLK_F_SEG_MAX: u64 = 1 << 2;
pub const VIRTIO_BLK_F_GEOMETRY: u64 = 1 << 4;
pub const VIRTIO_BLK_F_RO: u64 = 1 << 5;
pub const VIRTIO_BLK_F_BLK_SIZE: u64 = 1 << 6;
pub const VIRTIO_BLK_F_FLUSH: u64 = 1 << 9;
pub const VIRTIO_BLK_F_TOPOLOGY: u64 = 1 << 10;
pub const VIRTIO_BLK_F_MQ: u64 = 1 << 12;
pub const VIRTIO_BLK_F_DISCARD: u64 = 1 << 13;
pub const VIRTIO_BLK_F_WRITE_ZEROES: u64 = 1 << 14;

#[repr(C)]
#[derive(Debug)]
pub struct VirtioBlkReq {
    pub req_type: u32,
    pub reserved: u32,
    pub sector: u64,
    pub data: *mut u8,
    pub status: u8,
}

pub struct VirtioBlk {
    pub mmio: VirtioMmio,
    pub queue: Box<VirtQueue>,
}

impl From<VirtQueueError> for BlockDeviceError {
    fn from(_: VirtQueueError) -> Self {
        BlockDeviceError::QueueError
    }
}

impl VirtioBlk {
    // TODO: ここでbase_addressを受け取るとMMIO前提となってしまう。PCI等の対応
    pub fn new(mmio_device: VirtioMmio) -> Result<Self, VirtQueueError> {
        let vq = mmio_device.setup_virt_queue(VIRTIO_DEFAULT_INDEX)?;

        Ok(VirtioBlk {
            mmio: mmio_device,
            queue: vq,
        })
    }
}

impl BlockDevice for VirtioBlk {
    fn read_write_disk(
        &mut self,
        buf_address: *mut usize,
        sector: u64,
        count: usize,
        is_write: bool,
    ) -> Result<(), BlockDeviceError> {
        // make a request
        let req_type = if is_write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        let virtio_blk_req = VirtioBlkReq {
            req_type: req_type as u32,
            reserved: 0,
            sector,
            data: core::ptr::null_mut(),
            status: 0xff,
        };

        // setting Virtqueue
        let desc = &mut self.queue.vring.desc;
        desc[0].addr = core::ptr::addr_of!(virtio_blk_req) as u64;
        desc[0].len = size_of::<u32>() as u32 * 2 + size_of::<u64>() as u32;
        desc[0].flags = VRingDesc::VIRTQ_DESC_F_NEXT as u16;
        desc[0].next = 1;

        desc[1].addr = buf_address as u64;
        desc[1].len = (SECTOR_SIZE * count) as u32;
        desc[1].flags = VRingDesc::VIRTQ_DESC_F_NEXT as u16;
        if !is_write {
            desc[1].flags |= VRingDesc::VIRTQ_DESC_F_WRITE as u16;
        }
        desc[1].next = 2;

        // status field in VirtioBlkReq
        desc[2].addr = core::ptr::addr_of!(virtio_blk_req) as *const VirtioBlkReq as u64
            + (desc[0].len as usize + size_of::<*mut u32>()) as u64;
        desc[2].len = size_of::<u8>() as u32;
        desc[2].flags = VRingDesc::VIRTQ_DESC_F_WRITE as u16;
        desc[2].next = 0;

        // for i in 0..3 {
        //     println!("desc[{}]: addr: {:#x}, len: {}", i, desc[i].addr as usize, desc[i].len);
        // }

        self.queue.connect_to_avail_ring(0);
        self.queue.last_used_index = self.queue.last_used_index.wrapping_add(1);

        self.mmio.notify_to_device(VIRTIO_DEFAULT_INDEX);

        while self.queue.last_used_index
            != unsafe { core::ptr::read_volatile(&self.queue.vring.used.idx) }
        {}

        match virtio_blk_req.status as usize {
            VIRTIO_BLK_S_OK => return Ok(()),
            VIRTIO_BLK_S_IOERR => return Err(BlockDeviceError::IOError),
            VIRTIO_BLK_S_UNSUPP => return Err(BlockDeviceError::UnsupportedDevice),
            _ => unreachable!(),
        }
    }

    fn get_capacity(&self) -> usize {
        unsafe { core::ptr::read_volatile((self.mmio.base_address + 0x100) as *mut u64) as usize }
    }
}
