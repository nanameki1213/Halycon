#![no_std]
#![allow(dead_code)]

extern crate alloc;

use alloc::boxed::Box;
use core::{fmt, usize};

// analyze dtb and get mmio address
pub const VIRTIO_MMIO_DEFAULT_ADDRESS: usize = 0x10001000;
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
#[derive(Debug)]
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

    pub fn connect_to_avail_ring(&mut self, desc_idx: u16) {
        let idx = self.vring.avail.idx as usize;
        self.vring.avail.ring[idx % VIRTQ_ENTRY_NUM as usize] = desc_idx;
        self.vring.avail.idx = idx as u16 + 1;
    }
}

#[derive(Debug)]
pub enum VirtQueueError {
    UnsupportedVersion,
    InvalidQueue,
    AlreadyInUse,
}

impl fmt::Display for VirtQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => write!(f, "Virtio version is not supported."),
            Self::InvalidQueue => write!(f, "Virt Queue is invalid."),
            Self::AlreadyInUse => write!(f, "Virt Queue is already in use."),
        }
    }
}

#[derive(Clone, Copy)]
pub struct VirtioMmio {
    pub base_address: usize,
}

impl VirtioMmio {
    pub const fn new(base_address: usize) -> Self {
        VirtioMmio { base_address }
    }

    pub fn get_virtio_mmio(&self, offset: usize) -> u32 {
        unsafe {
            let addr = (self.base_address + offset) as *mut u32;
            core::ptr::read_volatile(addr)
        }
    }

    pub fn set_virtio_mmio(&self, offset: usize, value: u32) {
        unsafe {
            let addr = (self.base_address + offset) as *mut u32;
            core::ptr::write_volatile(addr, value);
        }
    }

    pub fn init_default_features(&self) {
        let features = self.get_device_features();
        self.set_driver_features(features);
        self.set_driver_ok();
    }

    pub fn get_device_features(&self) -> u64 {
        // 1. Reset the device.
        self.set_virtio_mmio(VIRTIO_MMIO_STATUS, 0x0);
        // 2. Set the ACKNOWLEDGE status bit
        self.set_virtio_mmio(VIRTIO_MMIO_STATUS, VIRTIO_MMIO_STATUS_ACKNOWLEDGE as u32);
        // 3. Set the DRIVER status bit
        let mut status = self.get_virtio_mmio(VIRTIO_MMIO_STATUS);
        status |= VIRTIO_MMIO_STATUS_DRIVER as u32;
        self.set_virtio_mmio(VIRTIO_MMIO_STATUS, status);

        self.set_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES_SEL, 0);
        let device_features_low = self.get_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES);

        self.set_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES_SEL, 1);
        let device_features_high = self.get_virtio_mmio(VIRTIO_MMIO_DEVICE_FEATURES);

        ((device_features_high as u64) << 32) | (device_features_low as u64)
    }

    pub fn set_driver_features(&self, features: u64) {
        // 4. Read device feature bits, and write the subset of feature bits understood
        //    by the OS and driver to the device.
        let mask = (1 << u32::BITS) - 1;
        let driver_features_low = features & mask;
        let driver_features_high = (features & (mask << u32::BITS)) >> u32::BITS;

        self.set_virtio_mmio(VIRTIO_MMIO_DRIVER_FEATURES_SEL, 0);
        self.set_virtio_mmio(VIRTIO_MMIO_DRIVER_FEATURES, driver_features_low as u32);

        self.set_virtio_mmio(VIRTIO_MMIO_DRIVER_FEATURES_SEL, 1);
        self.set_virtio_mmio(VIRTIO_MMIO_DRIVER_FEATURES, driver_features_high as u32);

        // 5. Set the FEATURES_OK status bit.
        let mut status = self.get_virtio_mmio(VIRTIO_MMIO_STATUS);
        status |= VIRTIO_MMIO_STATUS_FEATURES_OK as u32;
        self.set_virtio_mmio(VIRTIO_MMIO_STATUS, status);

        // 6. Re-read device status to ensure the FEATURES_OK bit is still set
        //    : otherwise, the device does not support our subset of features and the device is unusable.
        // status = self.get_virtio_mmio(VIRTIO_MMIO_STATUS);
        // if (status as usize & VIRTIO_MMIO_STATUS_FEATURES_OK) == 0 {
        //     println!("the device does not support subset of features and the device is unusable.");
        //     panic!();
        // }
    }

    pub fn set_driver_ok(&self) {
        // 8. Set the DRIVER_OK status bit. At this point the device is "live".
        let mut status = self.get_virtio_mmio(VIRTIO_MMIO_STATUS);
        status |= VIRTIO_MMIO_STATUS_DRIVER_OK as u32;
        self.set_virtio_mmio(VIRTIO_MMIO_STATUS, status);
    }

    pub fn setup_virt_queue(&self, index: u32) -> Result<Box<VirtQueue>, VirtQueueError> {
        let version = self.get_virtio_mmio(VIRTIO_MMIO_VERSION);
        if version != VIRTIO_VERSION as u32 {
            return Err(VirtQueueError::UnsupportedVersion);
        }
        // 1. Select the queue writing its index to QueueSel.
        self.set_virtio_mmio(VIRTIO_MMIO_QUEUE_SEL, index);
        // 2. Check if the queue is not already in use
        if self.get_virtio_mmio(VIRTIO_MMIO_QUEUE_READY) != 0 {
            // u-boot is already using the queue 0, so we overriding.
            // return Err(VirtQueueError::AlreadyInUse);
        }
        // 3. Read maxium queue size (number of elements) from QueueNumMax
        let max_size = self.get_virtio_mmio(VIRTIO_MMIO_QUEUE_MAX);
        if max_size == 0 {
            return Err(VirtQueueError::InvalidQueue);
        }
        // 4. Allocate and zero the queue memory
        let vq: Box<VirtQueue> = Box::new(VirtQueue::new());
        // 5. Notify the device about the queue size by writing the size to QueueNum
        self.set_virtio_mmio(VIRTIO_MMIO_QUEUE_NUM, VIRTQ_ENTRY_NUM as u32);
        // 6. Write physical addresses of the queue's Descriptor Area, Driver Area and Device Area
        let desc_address = vq.vring.desc.as_ptr() as u64;
        let avail_address = (&(vq.vring.avail) as *const VringAvail) as u64;
        let used_address = (&(vq.vring.used) as *const VRingUsed) as u64;
        const VIRTIO_MMIO_MASK: u64 = (1 << 32) - 1;
        self.set_virtio_mmio(
            VIRTIO_MMIO_DESC_LOW,
            (desc_address & VIRTIO_MMIO_MASK) as u32,
        );
        self.set_virtio_mmio(
            VIRTIO_MMIO_DESC_HIGH,
            ((desc_address >> 32) & VIRTIO_MMIO_MASK) as u32,
        );
        self.set_virtio_mmio(
            VIRTIO_MMIO_DRIVER_LOW,
            (avail_address & VIRTIO_MMIO_MASK) as u32,
        );
        self.set_virtio_mmio(
            VIRTIO_MMIO_DRIVER_HIGH,
            ((avail_address >> 32) & VIRTIO_MMIO_MASK) as u32,
        );
        self.set_virtio_mmio(
            VIRTIO_MMIO_DEVICE_LOW,
            (used_address & VIRTIO_MMIO_MASK) as u32,
        );
        self.set_virtio_mmio(
            VIRTIO_MMIO_DEVICE_HIGH,
            ((used_address >> 32) & VIRTIO_MMIO_MASK) as u32,
        );
        // 7. Write 0x1 to QueueReady
        self.set_virtio_mmio(VIRTIO_MMIO_QUEUE_READY, 0x1);

        Ok(vq)
    }

    pub fn notify_to_device(&self, index: u32) {
        self.set_virtio_mmio(VIRTIO_MMIO_QUEUE_NOTIFY, index);
    }

    pub fn is_queue_available(&self, index: u32) -> bool {
        self.set_virtio_mmio(VIRTIO_MMIO_QUEUE_SEL, index);
        self.get_virtio_mmio(VIRTIO_MMIO_QUEUE_MAX) != 0
    }
}

// Virtio MMIO によって設定されたQueue情報
#[repr(C)]
pub struct VirtioMmioRegister {
    pub queue_num: u32,
    pub queue_sel: u32,
    pub desc_address: u64,
    pub driver_address: u64,
    pub device_address: u64,
    pub status: u32,
    pub device_features_low: u32,
    pub device_features_high: u32,
    pub device_features_sel: u32,
    pub driver_features_low: u32,
    pub driver_features_high: u32,
    pub driver_features_sel: u32,
}

impl VirtioMmioRegister {
    pub const fn new() -> Self {
        VirtioMmioRegister {
            queue_num: 0,
            queue_sel: 0,
            desc_address: 0,
            driver_address: 0,
            device_address: 0,
            status: 0,
            device_features_low: 0,
            device_features_high: 0,
            device_features_sel: 0,
            driver_features_low: 0,
            driver_features_high: 0,
            driver_features_sel: 0,
        }
    }
}
