use crate::print;
use mmio_core::MmioHandler;

#[derive(Debug)]
pub struct Ns16550;

impl MmioHandler for Ns16550 {
    fn read(&self, offset: usize) -> usize {
        ns16550_get_by_offset(offset) as usize
    }

    fn write(&mut self, offset: usize, value: usize) {
        match offset {
            0x0 => {
                if let Some(ch) = char::from_u32(value as u32) {
                    ns16550_set_by_offset(offset, value as u8);
                    if ch == '\n' {
                        print!("[L1 VM] ");
                    }
                }
            }
            _ => ns16550_set_by_offset(offset, value as u8),
        }
    }
}

// Real Device
// TODO: Decouple host device handling from this code
// by introducing a host device package/abstraction.

const NS16550_ADDR: usize = 0x10000000;

fn ns16550_get_by_offset(offset: usize) -> u8 {
    let address = (NS16550_ADDR + offset) as *mut u8;

    unsafe { core::ptr::read_volatile(address) }
}

fn ns16550_set_by_offset(offset: usize, value: u8) {
    let address = (NS16550_ADDR + offset) as *mut u8;

    unsafe { core::ptr::write_volatile(address, value) }
}
