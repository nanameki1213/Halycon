#![no_std]

extern crate alloc;

use alloc::boxed::Box;

pub trait MmioAccess: Sized + Copy + From<u8> + Into<u64> {
    const SIZE: usize = core::mem::size_of::<Self>();
}

impl MmioAccess for u8 {}
impl MmioAccess for u16 {}
impl MmioAccess for u32 {}
impl MmioAccess for u64 {}

pub trait MmioHandler {
    fn read<T: MmioAccess>(&self, offset: usize) -> T;
    fn write<T: MmioAccess>(&self, offset: usize, value: T);
}

struct MmioEntry {
    pub address: usize,
    pub size: usize,
    pub handler: Box<dyn MmioHandler>,
}
