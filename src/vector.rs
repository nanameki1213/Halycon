use core::task::Context;
use core::u64;
use core::{arch::global_asm, usize};

use crate::mmio::ns16550;
use crate::paging;
use crate::cpu::*;
use crate::println;
use crate::sbi;
use crate::instruction;

pub const E_ILLEGAL_INSTRUCTION: usize = 2;
pub const E_LOAD_GUEST_PAGE_FAULT: usize = 21;
pub const E_STORE_AMO_GUEST_PAGE_FAULT: usize = 23;
pub const E_ENVIRONMENT_CALL_FROM_VS_MODE: usize = 10;

#[no_mangle]
#[link_section = ".data"]
pub static M_EXCEPTION: u8 = 0;

#[no_mangle]
#[link_section = ".data"]
pub static S_EXCEPTION: u8 = 1;

#[no_mangle]
#[link_section = ".data"]
pub static VS_EXCEPTION: u8 = 2;

global_asm!(
    "
.section .text
.global machine_vector_table
.balign 256
machine_vector_table:
    j machine_exception_handler 

.section .text
.global supervisor_vector_table
.balign 256
supervisor_vector_table:
    j supervisor_exception_handler 
    
.section .text
.global virtual_supervisor_vector_table
.balign 256
virtual_supervisor_vector_table:
    j virtual_supervisor_exception_handler 

.text
.extern M_EXCEPTION
.global machine_exception_handler 
.balign 256
machine_exception_handler:
    addi sp, sp, -8*31
    sw x0, 0*8(sp)
    sw x1, 1*8(sp)
    sw x2, 2*8(sp)
    sw x3, 3*8(sp)
    sw x4, 4*8(sp)
    sw x5, 5*8(sp)
    sw x6, 6*8(sp)
    sw x7, 7*8(sp)
    sw x8, 8*8(sp)
    sw x9, 9*8(sp)
    sw x10, 10*8(sp)
    sw x11, 11*8(sp)
    sw x12, 12*8(sp)
    sw x13, 13*8(sp)
    sw x14, 14*8(sp)
    sw x15, 15*8(sp)
    sw x16, 16*8(sp)
    sw x17, 17*8(sp)
    sw x18, 18*8(sp)
    sw x19, 19*8(sp)
    sw x20, 20*8(sp)
    sw x21, 21*8(sp)
    sw x22, 22*8(sp)
    sw x23, 23*8(sp)
    sw x24, 24*8(sp)
    sw x25, 25*8(sp)
    sw x26, 26*8(sp)
    sw x27, 27*8(sp)
    sw x28, 28*8(sp)
    sw x29, 29*8(sp)
    sw x30, 30*8(sp)
    sw x31, 31*8(sp)
    lb a0, M_EXCEPTION
    mv a1, sp
    call exception_handler
    lwu x0, 0*8(sp)
    lwu x1, 1*8(sp)
    lwu x2, 2*8(sp)
    lwu x3, 3*8(sp)
    lwu x4, 4*8(sp)
    lwu x5, 5*8(sp)
    lwu x6, 6*8(sp)
    lwu x7, 7*8(sp)
    lwu x8, 8*8(sp)
    lwu x9, 9*8(sp)
    lwu x10, 10*8(sp)
    lwu x11, 11*8(sp)
    lwu x12, 12*8(sp)
    lwu x13, 13*8(sp)
    lwu x14, 14*8(sp)
    lwu x15, 15*8(sp)
    lwu x16, 16*8(sp)
    lwu x17, 17*8(sp)
    lwu x18, 18*8(sp)
    lwu x19, 19*8(sp)
    lwu x20, 20*8(sp)
    lwu x21, 21*8(sp)
    lwu x22, 22*8(sp)
    lwu x23, 23*8(sp)
    lwu x24, 24*8(sp)
    lwu x25, 25*8(sp)
    lwu x26, 26*8(sp)
    lwu x27, 27*8(sp)
    lwu x28, 28*8(sp)
    lwu x29, 29*8(sp)
    lwu x30, 30*8(sp)
    lwu x31, 31*8(sp)
    addi sp, sp, 8*31
    mret

.text
.extern S_EXCEPTION
.global supervisor_exception_handler 
.balign 256
supervisor_exception_handler:
    addi sp, sp, -8*31
    sw x0, 0*8(sp)
    sw x1, 1*8(sp)
    sw x2, 2*8(sp)
    sw x3, 3*8(sp)
    sw x4, 4*8(sp)
    sw x5, 5*8(sp)
    sw x6, 6*8(sp)
    sw x7, 7*8(sp)
    sw x8, 8*8(sp)
    sw x9, 9*8(sp)
    sw x10, 10*8(sp)
    sw x11, 11*8(sp)
    sw x12, 12*8(sp)
    sw x13, 13*8(sp)
    sw x14, 14*8(sp)
    sw x15, 15*8(sp)
    sw x16, 16*8(sp)
    sw x17, 17*8(sp)
    sw x18, 18*8(sp)
    sw x19, 19*8(sp)
    sw x20, 20*8(sp)
    sw x21, 21*8(sp)
    sw x22, 22*8(sp)
    sw x23, 23*8(sp)
    sw x24, 24*8(sp)
    sw x25, 25*8(sp)
    sw x26, 26*8(sp)
    sw x27, 27*8(sp)
    sw x28, 28*8(sp)
    sw x29, 29*8(sp)
    sw x30, 30*8(sp)
    sw x31, 31*8(sp)
    lb a0, S_EXCEPTION
    mv a1, sp
    call exception_handler
    lwu x0, 0*8(sp)
    lwu x1, 1*8(sp)
    lwu x2, 2*8(sp)
    lwu x3, 3*8(sp)
    lwu x4, 4*8(sp)
    lwu x5, 5*8(sp)
    lwu x6, 6*8(sp)
    lwu x7, 7*8(sp)
    lwu x8, 8*8(sp)
    lwu x9, 9*8(sp)
    lwu x10, 10*8(sp)
    lwu x11, 11*8(sp)
    lwu x12, 12*8(sp)
    lwu x13, 13*8(sp)
    lwu x14, 14*8(sp)
    lwu x15, 15*8(sp)
    lwu x16, 16*8(sp)
    lwu x17, 17*8(sp)
    lwu x18, 18*8(sp)
    lwu x19, 19*8(sp)
    lwu x20, 20*8(sp)
    lwu x21, 21*8(sp)
    lwu x22, 22*8(sp)
    lwu x23, 23*8(sp)
    lwu x24, 24*8(sp)
    lwu x25, 25*8(sp)
    lwu x26, 26*8(sp)
    lwu x27, 27*8(sp)
    lwu x28, 28*8(sp)
    lwu x29, 29*8(sp)
    lwu x30, 30*8(sp)
    lwu x31, 31*8(sp)
    addi sp, sp, 8*31
    sret

.text
.extern VS_EXCEPTION
.global virtual_supervisor_exception_handler 
.balign 256
virtual_supervisor_exception_handler:
    addi sp, sp, -8*31
    sw x0, 0*8(sp)
    sw x1, 1*8(sp)
    sw x2, 2*8(sp)
    sw x3, 3*8(sp)
    sw x4, 4*8(sp)
    sw x5, 5*8(sp)
    sw x6, 6*8(sp)
    sw x7, 7*8(sp)
    sw x8, 8*8(sp)
    sw x9, 9*8(sp)
    sw x10, 10*8(sp)
    sw x11, 11*8(sp)
    sw x12, 12*8(sp)
    sw x13, 13*8(sp)
    sw x14, 14*8(sp)
    sw x15, 15*8(sp)
    sw x16, 16*8(sp)
    sw x17, 17*8(sp)
    sw x18, 18*8(sp)
    sw x19, 19*8(sp)
    sw x20, 20*8(sp)
    sw x21, 21*8(sp)
    sw x22, 22*8(sp)
    sw x23, 23*8(sp)
    sw x24, 24*8(sp)
    sw x25, 25*8(sp)
    sw x26, 26*8(sp)
    sw x27, 27*8(sp)
    sw x28, 28*8(sp)
    sw x29, 29*8(sp)
    sw x30, 30*8(sp)
    sw x31, 31*8(sp)
    lb a0, VS_EXCEPTION
    mv a1, sp
    call exception_handler
    lwu x0, 0*8(sp)
    lwu x1, 1*8(sp)
    lwu x2, 2*8(sp)
    lwu x3, 3*8(sp)
    lwu x4, 4*8(sp)
    lwu x5, 5*8(sp)
    lwu x6, 6*8(sp)
    lwu x7, 7*8(sp)
    lwu x8, 8*8(sp)
    lwu x9, 9*8(sp)
    lwu x10, 10*8(sp)
    lwu x11, 11*8(sp)
    lwu x12, 12*8(sp)
    lwu x13, 13*8(sp)
    lwu x14, 14*8(sp)
    lwu x15, 15*8(sp)
    lwu x16, 16*8(sp)
    lwu x17, 17*8(sp)
    lwu x18, 18*8(sp)
    lwu x19, 19*8(sp)
    lwu x20, 20*8(sp)
    lwu x21, 21*8(sp)
    lwu x22, 22*8(sp)
    lwu x23, 23*8(sp)
    lwu x24, 24*8(sp)
    lwu x25, 25*8(sp)
    lwu x26, 26*8(sp)
    lwu x27, 27*8(sp)
    lwu x28, 28*8(sp)
    lwu x29, 29*8(sp)
    lwu x30, 30*8(sp)
    lwu x31, 31*8(sp)
    addi sp, sp, 8*31
    sret
"
);

pub fn setup_vector() {
    extern "C" {
        static machine_vector_table: *const u8;
        static supervisor_vector_table: *const u8;
        static virtual_supervisor_vector_table: *const u8;
    }
    unsafe { set_mtvec((&machine_vector_table as *const _ as usize) as u64) }
    unsafe { set_stvec((&supervisor_vector_table as *const _ as usize) as u64) }
    unsafe { set_vstvec((&virtual_supervisor_vector_table as *const _ as usize) as u64) }
}

fn is_data_abort(scause: usize) -> bool {
    scause == E_STORE_AMO_GUEST_PAGE_FAULT || 
    scause == E_LOAD_GUEST_PAGE_FAULT
}

fn is_instruction_abort(scause: usize) -> bool {
    scause == E_ILLEGAL_INSTRUCTION ||
    scause == E_ENVIRONMENT_CALL_FROM_VS_MODE
}

#[no_mangle]
pub fn exception_handler(mode: u8, sp: usize) {
    if mode == M_EXCEPTION {
        println!("Exception from M-Mode has occured!");
        let mcause = get_mcause();
        println!("[info] mcause: {:#X}", mcause);
        println!("[info] mtval: {:#X}", get_mtval());

        if mcause == 20 {
            println!("[info] mtinst: {:#X}", get_mtinst());
        }

        panic!();
    } else if mode == VS_EXCEPTION {
        println!("Exception from VS-Mode has occured!");
        println!("[info] vscause: {:#X}", get_vscause());
        println!("[info] vstval: {:#X}", get_vstval());

        panic!();
    }
    
    let scause = get_scause() as usize;

    let contexts = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32)
    };
    if is_data_abort(scause) { // data abort
        data_abort_handler(scause, contexts);
    } else if is_instruction_abort(scause) { // instruction abort
        instruction_abort_handler(scause, contexts);
    }
    // next instruction
    let mut sepc = get_sepc();
    sepc += 4;
    set_sepc(sepc);
}

fn write_access(virtual_address: usize, value: u64) {
    if (ns16550::NS16550_ADDR..=ns16550::NS16550_ADDR + 0x100).contains(&(virtual_address)) {
            ns16550::write_ns16550(virtual_address - ns16550::NS16550_ADDR, value as u64);
    } else {
        println!("Exception from S-mode has occured!");
        println!("[info] virtual address: {:#X}", virtual_address);
        let physical_address = paging::resolve_address_stage2(virtual_address).unwrap();
        println!("[info] physical address: {:#X}", physical_address);
        panic!();
    }
}

fn read_access(virtual_address: usize, dst_register_idx: usize, registers: &mut [u64]) {
    if (ns16550::NS16550_ADDR..=ns16550::NS16550_ADDR + 0x100).contains(&(virtual_address)) {
            registers[dst_register_idx] = ns16550::read_ns16550(virtual_address - ns16550::NS16550_ADDR).unwrap();
    } else {
        println!("Exception from S-mode has occured!");
        println!("[info] virtual address: {:#X}", virtual_address);
        let physical_address = paging::resolve_address_stage2(virtual_address).unwrap();
        println!("[info] physical address: {:#X}", physical_address);
        panic!();
    }
}

fn data_abort_handler(scause: usize, registers: &mut [u64]) {
    match scause {
        E_STORE_AMO_GUEST_PAGE_FAULT => { // write access
            let stval = get_stval() as usize;
            let register_idx = (get_htinst() as u32 & instruction::RS2_MASK) >> instruction::RS2_OFFSET;
            let value = registers[register_idx as usize];
            write_access(stval, value);
        },
        E_LOAD_GUEST_PAGE_FAULT => {
            let stval = get_stval() as usize;
            let register_idx = (get_htinst() as u32 & instruction::RD_MASK) >> instruction::RD_OFFSET;
            read_access(stval, register_idx as usize, registers);
        },
        _ => {
            
        }
    };
}

fn instruction_abort_handler(scause: usize, registers: &mut [u64]) {
    match scause {
        E_ILLEGAL_INSTRUCTION => {
            
        },
        E_ENVIRONMENT_CALL_FROM_VS_MODE => {
            // TODO: 割り込み時のコンテキストをスタック上ではなくVM構造体に直接保存
            let a6 = registers[REGISTER_A6];
            let a7 = registers[REGISTER_A7];
            let mut sbi_ret = sbi::Sbiret {
                error: 0,
                value: 0
            };
            virtual_sbi(&mut sbi_ret, a7, a6);
            registers[REGISTER_A0] = sbi_ret.error; // a0
            registers[REGISTER_A1] = sbi_ret.value; // a1;
        },
        _ => {
            println!("Exception from S-mode has occured!");
            println!("[info] virtual address: {:#X}", get_sepc());
            let physical_address = paging::resolve_address_stage2(get_sepc() as usize).unwrap();
            println!("[info] physical address: {:#X}", physical_address);
            panic!();
        }
    };
}

fn virtual_sbi(sbi_ret: &mut sbi::Sbiret, eid: u64, fid: u64) {
    match fid {
        sbi::SBI_FID_PROBE_SBI_EXT => {
            if eid == sbi::SBI_EXT_BASE {
                sbi_ret.value = 1;
            }
        },
        sbi::SBI_FID_GET_SBI_IMPLEMENTATION_VERSION => {
            sbi_ret.value = 2;
        },
        _ => {
            println!("fid: {}", fid);
            panic!("unrecognized fid");
        }
    }
}
