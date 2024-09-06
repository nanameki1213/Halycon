use crate::cpu::*;
use core::arch::global_asm;

global_asm!(
    "
.section .text
.global vector_table
.balign 256
vector_table:
    j undefined_handler
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
    j external_handler
    j undefined_handler

undefined_handler:
    j undefined_handler

software_handler:
    j software_handler

timer_handler:
    j timer_handler

external_handler:
    j external_handler
"
);

pub fn setup_vector() {
    extern "C" {
        static vector_table: *const u8;
    }
    unsafe { set_mtvec(((&vector_table as *const _ as usize) | MTVEC_VECTORED) as u64) }
}
