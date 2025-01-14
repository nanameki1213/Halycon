#![feature(riscv_ext_intrinsics)]
#![no_std]
#![no_main]
#[macro_use]

mod cpu;
mod console;
mod memory;
mod paging;
mod vector;
mod virtio;
mod virtio_blk;
mod loader;
mod mmio {
    pub mod ns16550;
}

use crate::cpu::*;
use core::{arch::asm, usize};
use loader::load_bootloader;
use memory::*;
use paging::*;
use vector::setup_vector;
use virtio::{init_virtio_mmio, VIRTIO_DEFAULT_INDEX};
use virtio_blk::{init_virtio_blk, read_write_disk, SECTOR_SIZE};

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
extern "C" fn main() -> usize {
    println!("booting Halycon...");

    println!("[info] XLEN: {}", get_xlen_from_misa());

    let misa = get_misa();
    if (misa & (1 << MISA_EXTENSION_H_OFFSET)) == 0 {
        println!("this implimentesion is not support hypervisor extension.");
        return 1;
    }

    println!("[info] misa: {:#X}", misa);

    set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    println!("[setup] mie");

    setup_vector(); 
    println!("[setup] mtvec");
    println!("[setup] stvec");

    // let mut mstatus = get_mstatus();
    // mstatus |= (1 << MSTATUS_TVM_OFFSET) as u64;
    // set_mstatus(mstatus);

    let mut medeleg = get_medeleg();
    medeleg |= (1 << 20) as u64;
    medeleg |= (1 << 12) as u64;
    set_medeleg(medeleg);
    println!("[setup] medeleg: {:#X}", medeleg);

    let mut hedeleg = get_hedeleg();
    hedeleg |= (1 << 12) as u64;
    hedeleg |= (1 << 7) as u64;
    set_hedeleg(hedeleg);
    println!("[setup] hedeleg");

    let vm_address: fn() = vs_main;

    // 仮想マシンの領域のPMPを設定する;
    // let top_address = 0xF0000000 as usize;
    // let bottom_address = 0x80000000 as usize;
    // set_pmp(top_address, bottom_address, true, true, true);
    // println!("[setup] pmp: {:#X} ~ {:#X}", bottom_address, top_address);

    set_pmp_all_physical_address(true, true, true);
    println!("[setup] pmpaddr0: {:#X}", get_pmpaddr0());

    // let pmpcfg0 = get_pmpcfg0();
    // println!("[info] pmpcfg0: {:#X}", pmpcfg0);

    // let pmpcfg2 = get_pmpcfg2();
    // println!("[info] pmpcfg2: {:#X}", pmpcfg2);

    // let pmpaddr0 = get_pmpaddr0();
    // println!("[info] pmpaddr0: {:#X}", pmpaddr0);

    unsafe { init_allocation() };
    println!("[setup] allocater");

    let table_address = map_address_stage2(0x10000000, 0x10000000, 0xF0000000, DEFAULT_TABLE_LEVEL, true, true, true).expect("Failed to mapping");
    let mut hgatp = match DEFAULT_TABLE_LEVEL {
        3 => 0b1000 << 60,
        4 => 0b1001 << 60,
        5 => 0b1010 << 60,
        _ => unreachable!(),
    };
    hgatp |= (table_address >> 12) & SATP_PPN_MASK;
    set_hgatp(hgatp as u64);
    unsafe {
        core::arch::riscv64::hfence_gvma_all();
        core::arch::riscv64::hfence_vvma_all();
        core::arch::riscv64::sfence_vma_all();
    }

    let physical_vm_address = resolve_address_stage2(vm_address as usize).expect("Failed to resolve address");
    println!("[setup] stage2 paging");

    let menvcfg = get_menvcfg();
    let henvcfg = get_henvcfg();
    
    println!("[info] menvcfg: {:#X}", menvcfg);
    println!("[info] henvcfg: {:#X}", henvcfg);

    let stack_address = unsafe { allocate_memory(2, 0x1000).unwrap() + (2 << paging::PAGE_SHIFT) };
    println!("[info] stack_address: {:#X}", stack_address);
    println!("[info] vm virtual address: {:#X}", vs_main as u64);
    println!("[info] vm physical address: {:#X}", physical_vm_address);

    load_bootloader();

    println!("switch to guest");
    hs_to_vs(vm_address as usize, stack_address);
    // don't return to here
}

fn vs_main() {
    println!("Hello, World from Virtual Supervisor Mode!");

    loop {
        // unsafe { asm!("wfi") };
    }
}

fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize) -> ! {
    unsafe {
        asm!("
            csrs sstatus, {tmp1}
            csrs hstatus, {tmp2}
            csrw sepc, {entry_point}
            mv sp, {stack_pointer}
            sret", 
        tmp1 = in(reg) 0x100 as u64, // set sstatus.SPP
        tmp2 = in(reg) 0x80 as u64, // set hstatus.SPV
        stack_pointer = in(reg) vs_stack_pointer,
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
