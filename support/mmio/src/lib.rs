#![no_std]

extern crate alloc;

use alloc::boxed::Box;

pub trait MmioHandler {
    fn read(&self, offset: usize, width: usize) -> usize;
    fn write(&self, offset: usize, width: usize, value: usize);
}

struct MmioEntry {
    pub address: usize,
    pub size: usize,
    pub handler: Box<dyn MmioHandler>,
}
