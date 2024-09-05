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
use paging::{init_stage_2_paging, map_address_stage2, DEFAULT_TABLE_LEVEL};

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

    init_stage_2_paging(DEFAULT_TABLE_LEVEL);
    map_address_stage2(0x0, 0x0, 0x0, true, true);

    loop {}
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
