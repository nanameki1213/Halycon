use core::{arch::global_asm, usize};

use crate::paging;
use crate::cpu::*;
use crate::println;

pub const E_ILLEGAL_INSTRUCTION: usize = 2;
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
    addi sp, sp, -8*17
    sw ra, 0*8(sp)
    sw a0, 1*8(sp)
    sw a1, 2*8(sp)
    sw a2, 3*8(sp)
    sw a3, 4*8(sp)
    sw a4, 5*8(sp)
    sw a5, 6*8(sp)
    sw a6, 7*8(sp)
    sw a7, 8*8(sp)
    sw t0, 9*8(sp)
    sw t1, 10*8(sp)
    sw t2, 11*8(sp)
    sw t3, 12*8(sp)
    sw t4, 13*8(sp)
    sw t5, 14*8(sp)
    sw t6, 15*8(sp)
    sw s0, 16*8(sp)
    lb a0, M_EXCEPTION
    mv a1, sp
    call exception_handler
    lw ra, 0*8(sp)
    lw a0, 1*8(sp)
    lw a1, 2*8(sp)
    lw a2, 3*8(sp)
    lw a3, 4*8(sp)
    lw a4, 5*8(sp)
    lw a5, 6*8(sp)
    lw a6, 7*8(sp)
    lw a7, 8*8(sp)
    lw t0, 9*8(sp)
    lw t1, 10*8(sp)
    lw t2, 11*8(sp)
    lw t3, 12*8(sp)
    lw t4, 13*8(sp)
    lw t5, 14*8(sp)
    lw t6, 15*8(sp)
    lw s0, 16*8(sp)
    addi sp, sp, 8*17
    mret

.text
.extern S_EXCEPTION
.global supervisor_exception_handler 
.balign 256
supervisor_exception_handler:
    addi sp, sp, -8*17
    sw ra, 0*8(sp)
    sw a0, 1*8(sp)
    sw a1, 2*8(sp)
    sw a2, 3*8(sp)
    sw a3, 4*8(sp)
    sw a4, 5*8(sp)
    sw a5, 6*8(sp)
    sw a6, 7*8(sp)
    sw a7, 8*8(sp)
    sw t0, 9*8(sp)
    sw t1, 10*8(sp)
    sw t2, 11*8(sp)
    sw t3, 12*8(sp)
    sw t4, 13*8(sp)
    sw t5, 14*8(sp)
    sw t6, 15*8(sp)
    sw s0, 16*8(sp)
    lb a0, S_EXCEPTION
    mv a1, sp
    call exception_handler
    lw ra, 0*8(sp)
    lw a0, 1*8(sp)
    lw a1, 2*8(sp)
    lw a2, 3*8(sp)
    lw a3, 4*8(sp)
    lw a4, 5*8(sp)
    lw a5, 6*8(sp)
    lw a6, 7*8(sp)
    lw a7, 8*8(sp)
    lw t0, 9*8(sp)
    lw t1, 10*8(sp)
    lw t2, 11*8(sp)
    lw t3, 12*8(sp)
    lw t4, 13*8(sp)
    lw t5, 14*8(sp)
    lw t6, 15*8(sp)
    lw s0, 16*8(sp)
    addi sp, sp, 8*17
    sret

.text
.extern VS_EXCEPTION
.global virtual_supervisor_exception_handler 
.balign 256
virtual_supervisor_exception_handler:
    addi sp, sp, -8*17
    sw ra, 0*8(sp)
    sw a0, 1*8(sp)
    sw a1, 2*8(sp)
    sw a2, 3*8(sp)
    sw a3, 4*8(sp)
    sw a4, 5*8(sp)
    sw a5, 6*8(sp)
    sw a6, 7*8(sp)
    sw a7, 8*8(sp)
    sw t0, 9*8(sp)
    sw t1, 10*8(sp)
    sw t2, 11*8(sp)
    sw t3, 12*8(sp)
    sw t4, 13*8(sp)
    sw t5, 14*8(sp)
    sw t6, 15*8(sp)
    sw s0, 16*8(sp)
    lb a0, VS_EXCEPTION
    mv a1, sp
    call exception_handler
    lw ra, 0*8(sp)
    lw a0, 1*8(sp)
    lw a1, 2*8(sp)
    lw a2, 3*8(sp)
    lw a3, 4*8(sp)
    lw a4, 5*8(sp)
    lw a5, 6*8(sp)
    lw a6, 7*8(sp)
    lw a7, 8*8(sp)
    lw t0, 9*8(sp)
    lw t1, 10*8(sp)
    lw t2, 11*8(sp)
    lw t3, 12*8(sp)
    lw t4, 13*8(sp)
    lw t5, 14*8(sp)
    lw t6, 15*8(sp)
    lw s0, 16*8(sp)
    addi sp, sp, 8*17
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

#[no_mangle]
pub fn exception_handler(mode: u8, _sp: usize) {
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
    
    let scause = get_scause();

    if scause == E_ILLEGAL_INSTRUCTION as u64 {
        let sepc = get_sepc();
        let physical_address = paging::resolve_address_stage2(sepc as usize).unwrap();
        println!("Exception from S-Mode has occured!");
        println!("[info] virtual address: {:#X}", sepc);
        println!("[info] physical address: {:#X}", physical_address);
        println!("[info] scause: {:#X}", scause);
        println!("[info] stval: {:#X}", get_stval());

        panic!();
    } else if scause == E_STORE_AMO_GUEST_PAGE_FAULT as u64 {
        let stval = get_stval();
        let physical_address = paging::resolve_address_stage2(stval as usize).unwrap();
        println!("Exception from S-mode has occured!");
        println!("[info] virtual address: {:#X}", stval);
        println!("[info] physical address: {:#X}", physical_address);
        println!("[info] scause: {:#X}", scause);
    } else if scause == E_ENVIRONMENT_CALL_FROM_VS_MODE as u64 {
        panic!();
    } else {
        let sepc = get_sepc();
        let physical_address = paging::resolve_address_stage2(sepc as usize).unwrap();
        println!("Exception from S-mode has occured!");
        println!("[info] virtual address: {:#X}", sepc);
        println!("[info] physical address: {:#X}", physical_address);
        println!("[info] scause: {:#X}", scause);

        panic!();
    }
}

fn virtual_
