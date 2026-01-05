#![no_std]

pub const VIRTIO_VERSION: usize = 0x2;
pub const VIRTQ_ENTRY_NUM: u16 = 64;

pub const VIRTIO_MMIO_MAGIC_VALUE: usize = 0x74726976;

pub const VIRTIO_MMIO_MAGIC: usize = 0x00;
pub const VIRTIO_MMIO_VERSION: usize = 0x04;
pub const VIRTIO_MMIO_DEVICEID: usize = 0x08;
pub const VIRTIO_MMIO_VENDERID: usize = 0x0c;
pub const VIRTIO_MMIO_DEVICE_FEATURES: usize = 0x10;
pub const VIRTIO_MMIO_DEVICE_FEATURES_SEL: usize = 0x14;
pub const VIRTIO_MMIO_DRIVER_FEATURES: usize = 0x20;
pub const VIRTIO_MMIO_DRIVER_FEATURES_SEL: usize = 0x24;
pub const VIRTIO_MMIO_QUEUE_SEL: usize = 0x30;
pub const VIRTIO_MMIO_QUEUE_SIZE_MAX: usize = 0x34;
pub const VIRTIO_MMIO_QUEUE_SIZE: usize = 0x38;
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

pub const VIRTIO_NETWORK_DEVICE_ID: usize = 1;
pub const VIRTIO_BLOCK_DEVICE_ID: usize = 2;
