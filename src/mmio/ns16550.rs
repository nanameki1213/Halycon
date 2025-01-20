#![allow(dead_code)]

use core::usize;
use crate::println;

pub const NS16550_ADDR: usize = 0x10000000;
const NS16500_RBR: usize = 0x0;

pub fn read_ns16550(offset: usize) -> Result<u64, ()> {
    match offset {
        0x5 => Ok(0x60),
        _ => Err(()),
    }
}

pub fn write_ns16550(offset: usize, value: u64) {
    match offset {
        0x1 => {
            // 割り込みの許可/禁止処理をエミュレートする
            if value == 1 {
                println!("interrupt enable");
            } else {
                println!("interrupt disable");
            }
        },
        _ => {
            println!("offset: {:#X}, value: {:#X}", offset, value);
        } 
    }
}

pub fn putc(c: u8) {
    let reg = NS16550_ADDR;
    unsafe { core::ptr::write_volatile(reg as *mut u32, c as u32) }
}
