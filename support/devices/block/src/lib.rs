#![no_std]

pub mod virtio_blk;

use core::fmt;

pub const SECTOR_SIZE: usize = 512;

#[derive(Debug)]
pub enum BlockDeviceError {
    IOError,
    UnsupportedDevice,
    QueueError,
}

impl fmt::Display for BlockDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOError => write!(f, "Input/Output ERROR"),
            Self::UnsupportedDevice => write!(f, "Unsupported by device"),
            Self::QueueError => write!(f, "Queue Error"),
        }
    }
}

pub trait BlockDevice {
    fn read_write_disk(
        &mut self,
        buf_address: *mut usize,
        sector: u64,
        count: usize,
        is_write: bool,
    ) -> Result<(), BlockDeviceError>;

    fn get_capacity(&self) -> usize;
}
