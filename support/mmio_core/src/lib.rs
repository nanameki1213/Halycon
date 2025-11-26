#![no_std]

extern crate alloc;

use alloc::boxed::Box;

pub trait MmioHandler: Send {
    fn read(&self, offset: usize) -> usize;
    fn write(&self, offset: usize, value: usize);
}

pub struct MmioEntry {
    pub address: usize,
    pub size: usize,
    pub handler: Box<dyn MmioHandler>,
}

impl MmioEntry {
    pub const fn new(address: usize, size: usize, handler: Box<dyn MmioHandler>) -> Self {
        MmioEntry {
            address,
            size,
            handler,
        }
    }
}
