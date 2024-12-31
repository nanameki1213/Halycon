use crate::{cpu::*, println};
use core::{arch::global_asm, usize};

global_asm!(
    "
.section .text
.global machine_vector_table
.balign 256
machine_vector_table:
    j synchronous_exception_handler 
    j software_handler
    j software_handler
    j software_handler
    j undefined_handler
    j timer_handler
    j timer_handler
    j timer_handler
    j undefined_handler
    j external_handler
    j external_handler
    j external_handler
    j external_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j guest_page_fault_handler

.section .text
.global supervisor_vector_table
.balign 256
supervisor_vector_table:
    j synchronous_exception_handler 
    j software_handler
    j software_handler
    j software_handler
    j undefined_handler
    j timer_handler
    j timer_handler
    j timer_handler
    j undefined_handler
    j external_handler
    j external_handler
    j external_handler
    j external_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j guest_page_fault_handler

.section .text
.global virtual_supervisor_vector_table
.balign 256
virtual_supervisor_vector_table:
    j synchronous_exception_handler 
    j software_handler
    j software_handler
    j software_handler
    j undefined_handler
    j timer_handler
    j timer_handler
    j timer_handler
    j undefined_handler
    j external_handler
    j external_handler
    j external_handler
    j external_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j undefined_handler
    j guest_page_fault_handler

undefined_handler:
    j undefined_handler

software_handler:
    j software_handler

timer_handler:
    j timer_handler

external_handler:
    j external_handler

.text
.global synchronous_exception_handler
.balign 256
synchronous_exception_handler:
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
    mv a0, sp
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

guest_page_fault_handler:
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
    mv a0, sp
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
    unsafe { set_mtvec(((&machine_vector_table as *const _ as usize) | TVEC_VECTORED) as u64) }
    unsafe { set_stvec(((&supervisor_vector_table as *const _ as usize) | TVEC_VECTORED) as u64) }
    unsafe { set_vstvec(((&virtual_supervisor_vector_table as *const _ as usize) | TVEC_VECTORED) as u64) }
}

#[no_mangle]
pub fn exception_handler(stack_pointer: usize) {
    println!("sp: {:#X}", stack_pointer);
    panic!("synchronous exception.");
}
