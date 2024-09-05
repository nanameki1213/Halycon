#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#[macro_use]

mod cpu;
mod console;
mod paging;
mod vector;
mod mmio {
    pub mod ns16550;
}

use mmio::ns16550::putc;
use vector::setup_vector;

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

fn init() {}

#[no_mangle]
extern "C" fn main() -> ! {
    set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    setup_vector();

    println!("hello, world!");

    let mut mstatus = get_mstatus();
    mstatus = mstatus | MSTATUS_TVM_OFFSET as u64;

    set_mstatus(mstatus);

    loop {}
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
