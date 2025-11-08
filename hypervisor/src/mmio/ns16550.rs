#![allow(dead_code)]

use crate::{VIRTUAL_UART_DEVICE, print, println};
use core::char;

// TODO: get plic_addr from device tree
pub const NS16550_ADDR: usize = 0x10000000;
pub const NS16500_RBR: usize = 0x0;
pub const NS16500_IER: usize = 0x1;
pub const NS16550_LSR: usize = 0x5;

const NS16550_IER_RX_INTR: usize = 1 << 0;

const DEFAULT_FIFO_SIZE: usize = 16;

// TODO: make trait
#[derive(Debug)]
pub struct Uart {
    pub fifo: [u8; DEFAULT_FIFO_SIZE],
    pub head: usize,
    pub tail: usize,
}

impl Uart {
    pub const fn new() -> Self {
        Uart {
            fifo: [0; DEFAULT_FIFO_SIZE],
            head: 0,
            tail: 0,
        }
    }

    pub fn fifo_in(&mut self, c: u8) {
        if (self.tail + 1) % DEFAULT_FIFO_SIZE != self.head {
            self.fifo[self.tail] = c;
            self.tail = (self.tail + 1) % DEFAULT_FIFO_SIZE;
        } else {
            panic!("fifo is overflow")
        }
    }

    pub fn fifo_out(&mut self) -> u8 {
        if self.head != self.tail {
            let c = self.fifo[self.head];
            self.head = (self.head + 1) % DEFAULT_FIFO_SIZE;
            c
        } else {
            panic!("fifo is underflow")
        }
    }

    pub fn is_fifo_empty(&mut self) -> bool {
        self.head == self.tail
    }
}

pub fn emulate_read_ns16550(offset: usize) -> Result<u32, ()> {
    let is_empty = VIRTUAL_UART_DEVICE.lock().is_fifo_empty();

    match offset {
        NS16500_RBR => {
            // RBRを読みに来ているということはデータがあると思っている
            let c = VIRTUAL_UART_DEVICE.lock().fifo_out();
            Ok(c as u32)
        }
        NS16550_LSR => {
            if is_empty {
                Ok(0x60)
            } else {
                Ok(0x1)
            }
        }
        _ => Ok(0),
    }
}

pub fn emulate_write_ns16550(offset: usize, value: u32) {
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

pub fn uart_fifo_push(c: u8) {
    VIRTUAL_UART_DEVICE.lock().fifo_in(c);
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
