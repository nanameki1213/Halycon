#![feature(riscv_ext_intrinsics)]
#![no_std]
#![no_main]
#[macro_use]

mod cpu;
mod fdt;
mod aplic;
mod console;
mod instruction;
mod loader;
mod memory;
mod paging;
mod plic;
mod sbi;
mod vector;
mod virtio_blk;
mod vm;
mod mmio {
    pub mod ns16550;
    pub mod virtio;
}

use crate::cpu::*;
use core::{arch::asm, usize};
use memory::*;
use vector::setup_vector;
use core::str;

#[macro_export]
macro_rules! bitmask {
    ($high:expr,$low:expr) => {
        ((1 << (($high - $low) + 1)) - 1) << $low
    };
}

// fn intr_disable() {
//     set_mie(get_mie() & !(1 << MIE_MEIE_OFFSET));
// }

// allocが使えない環境下での文字列操作関数
unsafe fn hex_ptr_to_usize(ptr: *const u8) -> Result<usize, ()> {
    if ptr.is_null() {
        return Err(());
    }

    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }

    let slice = core::slice::from_raw_parts(ptr, len);
    let s = match str::from_utf8(slice) {
        Ok(s) => s,
        Err(_) => return Err(()),
    };

    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);

    usize::from_str_radix(s, 16).map_err(|_| ())
}

#[no_mangle]
extern "C" fn main(argc: usize, argv: *const *const u8) -> usize {
    if argc < 1 {
        panic!("dtb pointer not configured.");
    }

    let fdt_pointer: usize = unsafe {
        match hex_ptr_to_usize(*argv) {
            Ok(address) => address,
            Err(_) => panic!("fdt pointer not configured."),
        }
    };
    println!("booting Halycon...");

    println!("fdt pointer: {:#X}", fdt_pointer);

    unsafe {
        match fdt::parse_fdt(fdt_pointer) {
            Ok(()) => {},
            Err(msg) => panic!("{}", msg),
        }
    }

    let fdt_header = unsafe {
        match fdt::parse_fdt_header(fdt_pointer) {
            Ok(header) => header,
            Err(_) => panic!("cannnot read fdt header."),
        }
    };

    match fdt::check_fdt_header(&fdt_header) {
        Ok(()) => {},
        Err(msg) => panic!("{}", msg),
    }

    println!("{:?}", fdt_header);

    let xlen = get_xlen_from_misa();
    if xlen != 64 {
        println!("this implementation is not 64-bit.");
        return 1;
    }
    println!("[info] XLEN: {}", xlen);

    let misa = get_misa();
    if (misa & (1 << MISA_EXTENSION_H_OFFSET)) == 0 {
        println!("this implementation is not support hypervisor extension.");
        return 1;
    }

    println!("[info] misa: {:#X}", misa);

    // set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    // println!("[setup] mie");

    setup_vector();
    println!("[setup] mtvec");
    println!("[setup] stvec");

    // let mut mstatus = get_mstatus();
    // mstatus |= (1 << MSTATUS_TVM_OFFSET) as u64;
    // set_mstatus(mstatus);

    let mut medeleg = get_medeleg();
    medeleg |= (1 << 20) as u64;
    medeleg |= (1 << 12) as u64;
    medeleg |= (1 << 2) as u64;
    medeleg |= (1 << 23) as u64;
    medeleg |= (1 << 22) as u64;
    medeleg |= (1 << 21) as u64;
    medeleg |= (1 << 10) as u64;
    set_medeleg(medeleg);
    println!("[setup] medeleg: {:#X}", medeleg);

    let mut mideleg = get_mideleg();
    mideleg |= MIE_MEIE as u64;
    mideleg |= MIE_VSEIE as u64;
    mideleg |= XIE_SEIE as u64;
    set_mideleg(mideleg);
    println!("[setup] mideleg: {:#X}", mideleg);

    let mut hedeleg = get_hedeleg();
    hedeleg |= (1 << 12) as u64;
    hedeleg |= (1 << 7) as u64;
    set_hedeleg(hedeleg);
    println!("[setup] hedeleg");

    let mut mie = get_mie();
    mie |= MIE_MEIE as u64;
    set_mie(mie);
    println!("[setup] mie");

    let mut sie = get_sie();
    sie |= XIE_SEIE as u64;
    set_sie(sie);
    println!("[setup] sie");

    let mut hie = get_hie();
    hie |= XIE_SEIE as u64;
    set_hie(hie);

    let mut mstatus = get_mstatus();
    mstatus |= MSTATUS_SIE as u64;
    mstatus |= MSTATUS_MIE as u64;
    set_mstatus(mstatus);

    // 仮想マシンの領域のPMPを設定する;
    // let top_address = 0xF0000000 as usize;
    // let bottom_address = 0x80000000 as usize;
    // set_pmp(top_address, bottom_address, true, true, true);
    // println!("[setup] pmp: {:#X} ~ {:#X}", bottom_address, top_address);

    plic::init_plic();

    mmio::ns16550::ns16500_intr_receive_enable();

    set_pmp_all_physical_address(true, true, true);
    println!("[setup] pmpaddr0: {:#X}", get_pmpaddr0());

    unsafe { init_allocation() };
    println!("[setup] allocater");

    extern "C" {
        static _intr_stack_end: u8;
    }
    unsafe { println!("[info] intr stack pointer: {:#X}", &_intr_stack_end as *const u8 as usize); }

    // let stack_address = unsafe { allocate_memory(2, paging::PAGE_SIZE).unwrap() };
    let stack_address = 0x0;
    println!("[info] stack_address: {:#X}", stack_address);

    let vm = vm::create_vm();

    println!("switch to guest");
    unsafe {
        hs_to_vs(
            (*vm).get_entry_point() as usize,
            stack_address,
            (*vm).get_dtb_pointer(),
        )
    };
    // don't return to here
}

fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize, dtb_pointer: usize) -> ! {
    unsafe {
        asm!("
            csrs sstatus, {tmp1}
            csrs hstatus, {tmp2}
            csrw sepc, {entry_point}
            mv sp, {stack_pointer}
            mv a1, {dtb_pointer}
            sret", 
        tmp1 = in(reg) 0x100 as u64, // set sstatus.SPP
        tmp2 = in(reg) 0x80 as u64, // set hstatus.SPV
        stack_pointer = in(reg) vs_stack_pointer,
        entry_point = in(reg) vs_entry_point,
        dtb_pointer = in(reg) dtb_pointer,
        options(noreturn)
        )
    };
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
