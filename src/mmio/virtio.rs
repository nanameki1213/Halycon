#![allow(dead_code)]

extern crate alloc;

use crate::mmio::virtio;
use crate::paging::resolve_address_stage2;
use crate::println;
use crate::virtio_blk::{self, VirtioBlkReq};
use alloc::boxed::Box;
use core::usize;

// analyze dtb and get mmio address
pub const VIRTIO_MMIO_DEFAULT_ADDRESS: usize = 0x10001000;
pub static mut VIRTIO_MMIO_ADDRESS: usize = 0x10001000;
pub const VIRTIO_DEFAULT_INDEX: u32 = 0;

pub const VIRTIO_VERSION: usize = 0x2;
pub const VIRTQ_ENTRY_NUM: u16 = 64;

pub const VIRTIO_MMIO_MAGIC: usize = 0x00;
pub const VIRTIO_MMIO_VERSION: usize = 0x04;
pub const VIRTIO_MMIO_DEVICEID: usize = 0x08;
pub const VIRTIO_MMIO_VENDERID: usize = 0x0c;
pub const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x10;
pub const VIRTIO_MMIO_DEVICE_FEATURES_SEL: usize = 0x14;
pub const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x20;
pub const VIRTIO_MMIO_DRIVER_FEATURES_SEL: usize = 0x24;
pub const VIRTIO_MMIO_QUEUE_SEL: usize = 0x30;
pub const VIRTIO_MMIO_QUEUE_MAX: usize = 0x34;
pub const VIRTIO_MMIO_QUEUE_NUM: usize = 0x38;
pub const VIRTIO_MMIO_QUEUE_READY: usize = 0x44;
pub const VIRTIO_MMIO_QUEUE_NOTIFY: usize = 0x50;
pub const VIRTIO_MMIO_INTR_STATUS: usize = 0x60;
pub const VIRTIO_MMIO_INTR_ACK: usize = 0x64;
pub const VIRTIO_MMIO_STATUS: usize = 0x70;
pub const VIRTIO_MMIO_DESC_LOW: usize = 0x80;
pub const VIRTIO_MMIO_DESC_HIGH: usize = 0x84;
pub const VIRTIO_MMIO_DRIVER_LOW: usize = 0x90;
pub const VIRTIO_MMIO_DRIVER_HIGH: usize = 0x94;
pub const VIRTIO_MMIO_DEVICE_LOW: usize = 0xa0;
pub const VIRTIO_MMIO_DEVICE_HIGH: usize = 0xa4;
pub const VIRTIO_MMIO_CONFIG_GENERATION: usize = 0xfc;
pub const VIRTIO_MMIO_CONFIG: usize = 0x100;

pub const VIRTIO_MMIO_STATUS_ACKNOWLEDGE: usize = 1 << 0;
pub const VIRTIO_MMIO_STATUS_DRIVER: usize = 1 << 1;
pub const VIRTIO_MMIO_STATUS_DRIVER_OK: usize = 1 << 2;
pub const VIRTIO_MMIO_STATUS_FEATURES_OK: usize = 1 << 3;
pub const VIRTIO_MMIO_STATUS_DEVICE_NEEDS_RESET: usize = 1 << 6;
pub const VIRTIO_MMIO_STATUS_FAILED: usize = 1 << 7;

// virtio feature bits
pub const VIRTIO_F_INDIRECT_DESC: u64 = 1 << 28;
pub const VIRTIO_F_EVENT_IDX: u64 = 1 << 29;
pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
pub const VIRTIO_F_RING_RESET: u64 = 1 << 40;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VRingDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

impl VRingDesc {
    pub const VIRTQ_DESC_F_NEXT: usize = 1 << 0;
    pub const VIRTQ_DESC_F_WRITE: usize = 1 << 1;
    pub const VIRTQ_DESC_F_INDIRECT: usize = 1 << 2;

    pub const fn new() -> Self {
        VRingDesc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

#[repr(C)]
pub struct VringAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTQ_ENTRY_NUM as usize],
}

impl VringAvail {
    pub const fn new() -> Self {
        VringAvail {
            flags: 0,
            idx: 0,
            ring: [0; VIRTQ_ENTRY_NUM as usize],
        }
    }
}

#[derive(Debug)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VRingUsedElem {
    pub id: u32,
    pub len: u32,
}

impl VRingUsedElem {
    pub const fn new() -> Self {
        VRingUsedElem { id: 0, len: 0 }
    }
}

#[repr(C)]
pub struct VRingUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VRingUsedElem; VIRTQ_ENTRY_NUM as usize],
}

impl VRingUsed {
    pub const fn new() -> Self {
        VRingUsed {
            flags: 0,
            idx: 0,
            ring: [VRingUsedElem::new(); VIRTQ_ENTRY_NUM as usize],
        }
    }
}

#[repr(C)]
pub struct VRing {
    pub desc: [VRingDesc; VIRTQ_ENTRY_NUM as usize],
    pub avail: VringAvail,
    pub used: VRingUsed,
}

impl VRing {
    pub const fn new() -> Self {
        VRing {
            desc: [VRingDesc::new(); VIRTQ_ENTRY_NUM as usize],
            avail: VringAvail::new(),
            used: VRingUsed::new(),
        }
    }
}

#[repr(C)]
pub struct VirtQueue {
    pub vring: VRing,
    pub queue_index: u16,
    pub last_used_index: u16,
    pub last_avail_index: u16,
}

impl VirtQueue {
    pub const fn new() -> Self {
        VirtQueue {
            vring: VRing::new(),
            queue_index: 0,
            last_used_index: 0,
            last_avail_index: 0,
        }
    }
}

// Virtio MMIO によって設定されたQueue情報
#[repr(C)]
pub struct VirtQueueMmio {
    pub queue_num: u32,
    pub queue_sel: u32,
    pub desc_address: u64,
    pub driver_address: u64,
    pub device_address: u64,
    pub status: u8,
    pub device_features: u64,
    pub device_features_sel: u32,
    pub driver_features: u64,
    pub driver_features_sel: u32,
}

impl VirtQueueMmio {
    pub const fn new() -> Self {
        VirtQueueMmio {
            queue_num: 0,
            queue_sel: 0,
            desc_address: 0,
            driver_address: 0,
            device_address: 0,
            status: 0,
            device_features: 0,
            device_features_sel: 0,
            driver_features: 0,
            driver_features_sel: 0,
        }
    }
}

static mut VIRTQUEUE: VirtQueueMmio = VirtQueueMmio::new();
static mut VIRTUAL_VQ: *mut VirtQueue = core::ptr::null_mut();

#[inline(always)]
pub fn get_virtio_mmio(offset: usize) -> u32 {
    unsafe {
        let addr = (VIRTIO_MMIO_ADDRESS + offset) as *mut u32;
        core::ptr::read_volatile(addr)
    }
}

#[inline(always)]
pub fn set_virtio_mmio(offset: usize, value: u32) {
    unsafe {
        let addr = (VIRTIO_MMIO_ADDRESS + offset) as *mut u32;
        core::ptr::write_volatile(addr, value);
    }
}

pub fn init_virtio_mmio(index: u32) -> Result<*mut VirtQueue, ()> {
    let version = get_virtio_mmio(VIRTIO_MMIO_VERSION);
    if version != VIRTIO_VERSION as u32 {
        println!("virtio version is not compatible");
        return Err(());
    }
    // 1. Select the queue writing its index to QueueSel.
    set_virtio_mmio(VIRTIO_MMIO_QUEUE_SEL, index);
    // 2. Check if the queue is not already in use (ここではu-bootが先に制御しているので無視)
    // if get_virtio_mmio(VIRTIO_MMIO_QUEUE_READY) != 0 {
    //     println!("queue is already in use: {:#X}", get_virtio_mmio(VIRTIO_MMIO_QUEUE_READY));
    //     return Err(());
    // }
    // 3. Read maxium queue size (number of elements) from QueueNumMax
    let max_size = get_virtio_mmio(VIRTIO_MMIO_QUEUE_MAX);
    if max_size == 0 {
        println!("queue is invalid");
        return Err(());
    }
    // 4. Allocate and zero the queue memory
    let mut vq: Box<VirtQueue> = Box::new(VirtQueue::new());
    // 5. Notify the device about the queue size by writing the size to QueueNum
    set_virtio_mmio(VIRTIO_MMIO_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
    // 6. Write physical addresses of the queue's Descriptor Area, Driver Area and Device Area
    let desc_address = vq.vring.desc.as_ptr() as u64;
    let avail_address = (&(vq.vring.avail) as *const VringAvail) as u64;
    let used_address = (&(vq.vring.used) as *const VRingUsed) as u64;
    const VIRTIO_MMIO_MASK: u64 = (1 << 32) - 1;
    set_virtio_mmio(
        VIRTIO_MMIO_DESC_LOW,
        (desc_address & VIRTIO_MMIO_MASK) as u32,
    );
    set_virtio_mmio(
        VIRTIO_MMIO_DESC_HIGH,
        ((desc_address >> 32) & VIRTIO_MMIO_MASK) as u32,
    );
    set_virtio_mmio(
        VIRTIO_MMIO_DRIVER_LOW,
        (avail_address & VIRTIO_MMIO_MASK) as u32,
    );
    set_virtio_mmio(
        VIRTIO_MMIO_DRIVER_HIGH,
        ((avail_address >> 32) & VIRTIO_MMIO_MASK) as u32,
    );
    set_virtio_mmio(
        VIRTIO_MMIO_DEVICE_LOW,
        (used_address & VIRTIO_MMIO_MASK) as u32,
    );
    set_virtio_mmio(
        VIRTIO_MMIO_DEVICE_HIGH,
        ((avail_address >> 32) & VIRTIO_MMIO_MASK) as u32,
    );
    // 7. Write 0x1 to QueueReady
    set_virtio_mmio(VIRTIO_MMIO_QUEUE_READY, 0x1);

    Ok(vq.as_mut())
}

pub fn connect_to_avail_ring(queue: &mut VirtQueue, desc_idx: u16) {
    let idx = queue.vring.avail.idx as usize;
    queue.vring.avail.ring[idx % VIRTQ_ENTRY_NUM as usize] = desc_idx;
    queue.vring.avail.idx = idx as u16 + 1;
}

pub fn notify_to_device(index: u32) {
    set_virtio_mmio(VIRTIO_MMIO_QUEUE_NOTIFY, index);
}

pub fn is_queue_available(index: u32) -> bool {
    set_virtio_mmio(VIRTIO_MMIO_QUEUE_SEL, index);
    get_virtio_mmio(VIRTIO_MMIO_QUEUE_MAX) != 0
}

pub fn emulate_read_virtio(offset: usize) -> Result<u32, ()> {
    let address = unsafe { (virtio::VIRTIO_MMIO_ADDRESS + offset) as *mut u32 };

    let mut value = unsafe { core::ptr::read_volatile(address) };

    match offset {
        VIRTIO_MMIO_VERSION => unsafe {
            VIRTQUEUE.device_features = virtio_blk::VIRTIO_BLK_F_SEG_MAX
                | virtio_blk::VIRTIO_BLK_F_GEOMETRY
                | virtio_blk::VIRTIO_BLK_F_BLK_SIZE
                | virtio_blk::VIRTIO_BLK_F_FLUSH
                | virtio_blk::VIRTIO_BLK_F_TOPOLOGY
                | virtio_blk::VIRTIO_BLK_F_DISCARD
                | virtio_blk::VIRTIO_BLK_F_WRITE_ZEROES
                | VIRTIO_F_INDIRECT_DESC
                | VIRTIO_F_EVENT_IDX
                | VIRTIO_F_VERSION_1
                | VIRTIO_F_RING_RESET;
        },
        VIRTIO_MMIO_QUEUE_READY => {
            value = VIRTIO_DEFAULT_INDEX;
        }
        VIRTIO_MMIO_STATUS => unsafe {
            value = VIRTQUEUE.status as u32;
        },
        VIRTIO_MMIO_DEVICE_FEATURES => unsafe {
            let shift = VIRTQUEUE.device_features_sel * 32;
            value = (VIRTQUEUE.device_features >> shift) as u32;
        },
        _ => {}
    }

    println!("read: {:#X}, {:#X}", offset, value);
    Ok(value)
}

pub fn emulate_write_virtio(offset: usize, value: u32) {
    println!("write: {:#X}, {:#X}", offset, value);

    match offset {
        VIRTIO_MMIO_QUEUE_NOTIFY => unsafe {
            if value != VIRTQUEUE.queue_sel {
                println!("invalid queue num");
                return;
            }

            let desc_address = resolve_address_stage2(VIRTQUEUE.desc_address as usize).unwrap();
            let desc_ring = &mut *core::ptr::slice_from_raw_parts_mut(
                desc_address as *mut VRingDesc,
                VIRTQ_ENTRY_NUM as usize,
            );
            let request_address = resolve_address_stage2(desc_ring[0].addr as usize).unwrap();
            let data_address = resolve_address_stage2(desc_ring[1].addr as usize).unwrap();
            let status_address =
                resolve_address_stage2(desc_ring[2].addr as usize).unwrap() as *mut u8;
            let virtio_blk_req = &mut *(request_address as *mut virtio_blk::VirtioBlkReq);

            println!("desc[0]: {:?}", desc_ring[0]);
            println!("desc[1]: {:?}", desc_ring[1]);
            println!("desc[2]: {:?}", desc_ring[2]);

            // println!("virtio: {:?}", virtio_blk_req);

            if desc_ring[1].flags & VRingDesc::VIRTQ_DESC_F_WRITE as u16 != 0 {
                let mut req = VirtioBlkReq {
                    req_type: 0,
                    reserved: 0,
                    sector: 0,
                    data: [0; 512],
                    status: 0,
                };
                virtio_blk::read_write_disk(
                    &mut *VIRTUAL_VQ,
                    data_address as *mut usize,
                    &mut req,
                    virtio_blk_req.sector,
                    false,
                );
            }

            let device_address = resolve_address_stage2(VIRTQUEUE.device_address as usize).unwrap();
            let used_ring = &mut *(device_address as *mut VRingUsed);
            used_ring.idx += 1;

            core::ptr::write_volatile(status_address, virtio_blk::VIRTIO_BLK_S_OK as u8);
        },
        VIRTIO_MMIO_QUEUE_READY => unsafe {
            virtio::VIRTIO_MMIO_ADDRESS = 0x10003000;
            virtio_blk::init_virtio_blk();
            VIRTUAL_VQ = init_virtio_mmio(VIRTIO_DEFAULT_INDEX).unwrap();
        },
        VIRTIO_MMIO_QUEUE_NUM => unsafe {
            VIRTQUEUE.queue_num = value;
        },
        VIRTIO_MMIO_QUEUE_SEL => unsafe {
            VIRTQUEUE.queue_sel = value;
        },
        VIRTIO_MMIO_DEVICE_FEATURES_SEL => unsafe {
            VIRTQUEUE.device_features_sel = value;
        },
        VIRTIO_MMIO_DRIVER_FEATURES_SEL => unsafe {
            VIRTQUEUE.driver_features_sel = value;
        },
        VIRTIO_MMIO_DRIVER_FEATURES => unsafe {
            let shift = VIRTQUEUE.driver_features_sel * 32;
            VIRTQUEUE.driver_features = ((value as u64) << shift) as u64;
        },
        VIRTIO_MMIO_STATUS_FEATURES_OK => unsafe {
            VIRTQUEUE.status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u8;
        },
        VIRTIO_MMIO_STATUS => unsafe {
            VIRTQUEUE.status = value as u8;
        },
        VIRTIO_MMIO_DESC_LOW => unsafe {
            VIRTQUEUE.desc_address = value as u64;
        },
        VIRTIO_MMIO_DRIVER_LOW => unsafe {
            VIRTQUEUE.driver_address = value as u64;
        },
        VIRTIO_MMIO_DEVICE_LOW => unsafe {
            VIRTQUEUE.device_address = value as u64;
        },
        _ => {}
    }
}
