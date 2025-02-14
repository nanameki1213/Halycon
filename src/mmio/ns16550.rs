#![allow(dead_code)]

use crate::print;
use core::char;
use core::usize;

// TODO: get plic_addr from device tree
pub const NS16550_ADDR: usize = 0x10000000;
const NS16500_RBR: usize = 0x0;
const NS16500_IER: usize = 0x1;

const NS16550_IER_RX_INTR: usize = 1 << 0;

pub fn read_ns16550(offset: usize) -> Result<u64, ()> {
    match offset {
        0x5 => Ok(0x60),
        _ => Ok(0),
    }
}

pub fn write_ns16550(offset: usize, value: u64) {
    match offset {
        0x0 => {
            if let Some(ch) = char::from_u32(value as u32) {
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
    let ier_address = (NS16550_ADDR + NS16500_IER) as *mut u32;

    let ier = unsafe {
        core::ptr::read_volatile(ier_address)
    };

    unsafe {
        core::ptr::write_volatile(ier_address, ier | NS16550_IER_RX_INTR as u32);
    }
}

pub fn putc(c: u8) {
    let reg = NS16550_ADDR;
    unsafe { core::ptr::write_volatile(reg as *mut u32, c as u32) }
}
