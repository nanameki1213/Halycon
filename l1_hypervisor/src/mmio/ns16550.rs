#![allow(dead_code)]

use crate::print;
use core::char;

// TODO: get plic_addr from device tree
pub const NS16550_ADDR: usize = 0x10000000;
pub const NS16500_RBR: usize = 0x0;
pub const NS16500_IER: usize = 0x1;
pub const NS16550_LSR: usize = 0x5;

const NS16550_IER_RX_INTR: usize = 1 << 0;

pub fn read_ns16550(offset: usize) -> Result<u64, ()> {
    match offset {
        0x5 => Ok(0x60),
        _ => Ok(0),
    }
}

#[unsafe(no_mangle)]
pub fn write_ns16550(offset: usize, value: u32) {
    match offset {
        0x0 => {
            if let Some(ch) = char::from_u32(value) {
                if ch == '\n' {
                    print!("\r");
                }
                print!("{}", ch);
            }
        }
        0x1 => {
            // 割り込みの許可/禁止処理をエミュレートする
        }
        _ => {}
    }
}

pub fn ns16500_intr_receive_enable() {
    let ier_address = (NS16550_ADDR + NS16500_IER) as *mut u8;

    let ier = unsafe { core::ptr::read_volatile(ier_address) };

    unsafe { core::ptr::write_volatile(ier_address, ier | NS16550_IER_RX_INTR as u8) }
}

pub fn ns16550_get_by_offset(offset: usize) -> u8 {
    let address = (NS16550_ADDR + offset) as *mut u8;

    unsafe { core::ptr::read_volatile(address) }
}

pub fn ns16550_set_by_offset(offset: usize, value: u8) {
    let address = (NS16550_ADDR + offset) as *mut u8;

    unsafe { core::ptr::write_volatile(address, value) }
}

pub fn putc(c: u8) {
    let reg = NS16550_ADDR;
    unsafe { core::ptr::write_volatile(reg as *mut u32, c as u32) }
}
