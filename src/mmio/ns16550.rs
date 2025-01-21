#![allow(dead_code)]

use core::char;
use core::usize;
use crate::println;
use crate::print;

pub const NS16550_ADDR: usize = 0x10000000;
const NS16500_RBR: usize = 0x0;

pub fn read_ns16550(offset: usize) -> Result<u64, ()> {
    match offset {
        0x5 => {
            println!("[read] offset: {:#X}", offset);
            Ok(0x60)
        },
        _ => {
            println!("[read] offset: {:#X}", offset);
            Ok(0)
        }
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
        },
        0x1 => {
            // 割り込みの許可/禁止処理をエミュレートする
            if value == 1 {
                println!("interrupt enable");
            } else {
                println!("interrupt disable");
            }
        },
        _ => {
            println!("[write] offset: {:#X}, value: {:#X}", offset, value);
        } 
    }
}

pub fn putc(c: u8) {
    let reg = NS16550_ADDR;
    unsafe { core::ptr::write_volatile(reg as *mut u32, c as u32) }
}
