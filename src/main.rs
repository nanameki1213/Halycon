#![feature(const_mut_refs)]
#![feature(const_refs_to_static)]
#![feature(naked_functions)]
#![no_std]
#![no_main]

#[macro_use]
mod cpu;
mod vector;
mod console;
// mod paging;
mod mmio {
    pub mod ns16550;
}

use core::arch::asm;
use core::usize;
use core::ptr;

use vector::setup_vector;
use mmio::ns16550::putc;

use crate::cpu::*;

#[macro_export]
macro_rules! bitmask {
    ($high:expr,$low:expr) => {
        ((1 << (($high - $low) + 1)) - 1) << $low
    };
}

fn intr_disable() {
    set_mie(get_mie() & !(1 << MIE_MEIE_OFFSET));
}

fn init() {
    
}

extern "C" {
    static mut __stack_top: u8;
}

#[no_mangle]
#[link_section = ".main"]
extern "C" fn main() -> ! {
    set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    setup_vector();

    println!("hello, world!");

    unreachable!();
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
