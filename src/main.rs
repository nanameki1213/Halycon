#![no_std]
#![no_main]
#![feature(core_intrinsics)]
#[macro_use]

mod cpu;
mod console;
mod memory;
mod paging;
mod vector;
mod mmio {
    pub mod ns16550;
}

use crate::cpu::*;
use core::arch::asm;
use memory::{allocate_memory, init_allocation, set_pmp};
use paging::{init_stage_2_paging, map_address_stage2, DEFAULT_TABLE_LEVEL};
use vector::setup_vector;

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
    let misa = get_misa();

    if (misa & (1 << MISA_EXTENSION_H_OFFSET)) == 0 {
        println!("this implimentesion is not support hypervisor extension.");
        return;
    }

    println!("misa: {:#X}", misa);

    set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    setup_vector();

    println!("hello, world!");

    let mut mstatus = get_mstatus();
    mstatus |= (1 << MSTATUS_TVM_OFFSET) as u64;

    set_mstatus(mstatus);

    let mut medeleg = get_medeleg();
    medeleg |= (1 << 20) as u64;

    set_medeleg(medeleg);

    let vm_address: fn() = vs_main;

    // 仮想マシンの領域のPMPを設定する;
    set_pmp(vm_address as usize + 0x1000, vm_address as usize, true, true, true);

    println!("mstatus: {:#X}", mstatus);

    let pmpcfg0 = get_pmpcfg0();
    println!("pmpcfg0: {:#X}", pmpcfg0);

    let pmpcfg2 = get_pmpcfg2();
    println!("pmpcfg2: {:#X}", pmpcfg2);

    let pmpaddr0 = get_pmpaddr0();
    println!("pmpaddr0: {:#X}", pmpaddr0);

    unsafe { init_allocation() };
    init_stage_2_paging(DEFAULT_TABLE_LEVEL);
    hfence();

    map_address_stage2(0x80000000, 0x80000000, 0x10000000, true, true).expect("Failed to mapping");

    let stack_address = unsafe { allocate_memory(2, 0x1000).unwrap() + (2 << paging::PAGE_SHIFT) };
    println!("vs_main addr: {:#X}", vm_address as usize);

    hs_to_vs(vm_address as usize, stack_address);
    loop {}
}

fn vs_main() {
    println!("Hello, World from Virtual Supervisor Mode!");

    loop {
        // unsafe { asm!("wfi") };
    }
}

fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize) {
    unsafe {
        asm!("
            csrs sstatus, {tmp1}
            csrs hstatus, {tmp2}
            csrw sepc, {entry_point}
            sret", 
        tmp1 = in(reg) 0x100 as u64, // set sstatus.SPP
        tmp2 = in(reg) 0x80 as u64, // set hstatus.SPV
//        stack_pointer = in(reg) vs_stack_pointer,
        entry_point = in(reg) vs_entry_point,
        options(noreturn)
        )
    };
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
