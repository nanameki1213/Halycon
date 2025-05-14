#![allow(dead_code)]

use core::usize;

use crate::paging::{resolve_address_stage2, PAGE_SIZE};
use crate::{allocate_memory, println};

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

#[repr(C)]
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
}

#[repr(C)]
pub struct VringAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTQ_ENTRY_NUM as usize],
}

#[derive(Debug)]
#[repr(C)]
pub struct VRingUsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VRingUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VRingUsedElem; VIRTQ_ENTRY_NUM as usize],
}

#[repr(C)]
pub struct VRing {
    pub desc: [VRingDesc; VIRTQ_ENTRY_NUM as usize],
    pub avail: VringAvail,
    pub used: VRingUsed,
}

#[repr(C)]
pub struct VirtQueue {
    pub vring: VRing,
    pub queue_index: u16,
    pub last_used_index: u16,
    pub last_avail_index: u16,
}

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
    let vq = unsafe { &mut *(allocate_memory(1, PAGE_SIZE).unwrap() as *mut VirtQueue) };
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

    Ok(vq)
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

const VIRTIO_MMIO_EMULATE_OFFSET: usize = 0x2000;

pub fn emulate_read_virtio(offset: usize) -> Result<u32, ()> {
    let address = (VIRTIO_MMIO_DEFAULT_ADDRESS + VIRTIO_MMIO_EMULATE_OFFSET + offset) as *mut u32;

    let value = unsafe {
        core::ptr::read_volatile(address)
    };

    println!("read: {:#X}, {:#X}", offset, value);
    Ok(value)
}

pub fn emulate_write_virtio(offset: usize, value: u32) {
    println!("write: {:#X}, {:#X}", offset, value);

    let address = (VIRTIO_MMIO_DEFAULT_ADDRESS + VIRTIO_MMIO_EMULATE_OFFSET + offset) as *mut u32;

    match offset {
        VIRTIO_MMIO_DESC_LOW | VIRTIO_MMIO_DEVICE_LOW | VIRTIO_MMIO_DRIVER_LOW => {
            unsafe {
                core::ptr::write_volatile(address, resolve_address_stage2(value as usize).unwrap() as u32);
            }
        },
        _ => {
            unsafe {
                core::ptr::write_volatile(address, value);
            }
        }
    }
}
