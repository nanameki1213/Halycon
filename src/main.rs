#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#[macro_use]

mod cpu;
mod console;
mod paging;
mod vector;
mod memory;
mod mmio {
    pub mod ns16550;
}

use vector::setup_vector;
use paging::{init_stage_2_paging, map_address_stage2, DEFAULT_TABLE_LEVEL};
use memory::{init_allocation, allocate_memory};
use core::arch::asm;
use crate::cpu::*;

#[macro_export]
macro_rules! bitmask {
    ($high:expr,$low:expr) => {
        ((1 << (($high - $low) + 1)) - 1) << $low
    };
}

// fn intr_disable() {
//     set_mie(get_mie() & !(1 << MIE_MEIE_OFFSET));
// }

#[no_mangle]
extern "C" fn main() {

    let mut misa = get_misa();
    misa |= 1 << MISA_EXTENSION_H_OFFSET;

    set_misa(misa);

    set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    setup_vector();

    println!("hello, world!");

    let mut mstatus = get_mstatus();
    mstatus = mstatus | MSTATUS_TVM_OFFSET as u64;

    set_mstatus(mstatus);

    unsafe {
        init_allocation()
    };
    init_stage_2_paging(DEFAULT_TABLE_LEVEL);

    map_address_stage2(0x80000000, 0x80000000, 0x10000000, true, true).expect("Failed to mapping");

    let func: fn() = vs_main;

    let stack_address = unsafe {
        allocate_memory(2).unwrap() + (2 << paging::PAGE_SHIFT)
    };
    
//    hs_to_vs(func as usize, stack_address);

}

fn vs_main() {
    println!("Hello, World from Virtual Supervisor Mode!");

    loop {
        unsafe {
            asm!("wfi")
        };
    }
}

// fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize) {
//     unsafe {
//         asm!("
//             csrw sstatus, {tmp}
//             la sp, {stack_pointer}
//             csrw sepc, {entry_point}
//             sret", 
//         tmp = in(reg) 0,
//         stack_pointer = in(reg) vs_stack_pointer,
//         entry_point = in(reg) vs_entry_point,
//         options(noreturn)
//         )
//     };
// }

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
