use core::{arch::global_asm, usize};

use crate::cpu::*;
use crate::instruction;
use crate::mmio::{ns16550, virtio};
use crate::paging;
use crate::plic;
use crate::println;
use crate::sbi;

pub const E_ILLEGAL_INSTRUCTION: usize = 2;
pub const E_LOAD_GUEST_PAGE_FAULT: usize = 21;
pub const E_VIRTUAL_INSTRUCTION: usize = 22;
pub const E_STORE_AMO_GUEST_PAGE_FAULT: usize = 23;
pub const E_ENVIRONMENT_CALL_FROM_VS_MODE: usize = 10;

pub const INTERRUPT_ID: usize = 1 << (MXLEN - 1);
pub const I_MACHINE_EXTERNAL: usize = 11 | INTERRUPT_ID;

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
    sd x0, 0*8(sp)
    sd x1, 1*8(sp)
    sd x2, 2*8(sp)
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    sd x5, 5*8(sp)
    sd x6, 6*8(sp)
    sd x7, 7*8(sp)
    sd x8, 8*8(sp)
    sd x9, 9*8(sp)
    sd x10, 10*8(sp)
    sd x11, 11*8(sp)
    sd x12, 12*8(sp)
    sd x13, 13*8(sp)
    sd x14, 14*8(sp)
    sd x15, 15*8(sp)
    sd x16, 16*8(sp)
    sd x17, 17*8(sp)
    sd x18, 18*8(sp)
    sd x19, 19*8(sp)
    sd x20, 20*8(sp)
    sd x21, 21*8(sp)
    sd x22, 22*8(sp)
    sd x23, 23*8(sp)
    sd x24, 24*8(sp)
    sd x25, 25*8(sp)
    sd x26, 26*8(sp)
    sd x27, 27*8(sp)
    sd x28, 28*8(sp)
    sd x29, 29*8(sp)
    sd x30, 30*8(sp)
    sd x31, 31*8(sp)
    lb a0, M_EXCEPTION
    mv a1, sp
    call exception_handler
    ld x0, 0*8(sp)
    ld x1, 1*8(sp)
    ld x2, 2*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    ld x5, 5*8(sp)
    ld x6, 6*8(sp)
    ld x7, 7*8(sp)
    ld x8, 8*8(sp)
    ld x9, 9*8(sp)
    ld x10, 10*8(sp)
    ld x11, 11*8(sp)
    ld x12, 12*8(sp)
    ld x13, 13*8(sp)
    ld x14, 14*8(sp)
    ld x15, 15*8(sp)
    ld x16, 16*8(sp)
    ld x17, 17*8(sp)
    ld x18, 18*8(sp)
    ld x19, 19*8(sp)
    ld x20, 20*8(sp)
    ld x21, 21*8(sp)
    ld x22, 22*8(sp)
    ld x23, 23*8(sp)
    ld x24, 24*8(sp)
    ld x25, 25*8(sp)
    ld x26, 26*8(sp)
    ld x27, 27*8(sp)
    ld x28, 28*8(sp)
    ld x29, 29*8(sp)
    ld x30, 30*8(sp)
    ld x31, 31*8(sp)
    addi sp, sp, 8*31
    mret

.text
.extern S_EXCEPTION
.global supervisor_exception_handler 
.balign 256
supervisor_exception_handler:
    addi sp, sp, -8*31
    sd x0, 0*8(sp)
    sd x1, 1*8(sp)
    sd x2, 2*8(sp)
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    sd x5, 5*8(sp)
    sd x6, 6*8(sp)
    sd x7, 7*8(sp)
    sd x8, 8*8(sp)
    sd x9, 9*8(sp)
    sd x10, 10*8(sp)
    sd x11, 11*8(sp)
    sd x12, 12*8(sp)
    sd x13, 13*8(sp)
    sd x14, 14*8(sp)
    sd x15, 15*8(sp)
    sd x16, 16*8(sp)
    sd x17, 17*8(sp)
    sd x18, 18*8(sp)
    sd x19, 19*8(sp)
    sd x20, 20*8(sp)
    sd x21, 21*8(sp)
    sd x22, 22*8(sp)
    sd x23, 23*8(sp)
    sd x24, 24*8(sp)
    sd x25, 25*8(sp)
    sd x26, 26*8(sp)
    sd x27, 27*8(sp)
    sd x28, 28*8(sp)
    sd x29, 29*8(sp)
    sd x30, 30*8(sp)
    sd x31, 31*8(sp)
    lb a0, S_EXCEPTION
    mv a1, sp
    call exception_handler
    ld x0, 0*8(sp)
    ld x1, 1*8(sp)
    ld x2, 2*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    ld x5, 5*8(sp)
    ld x6, 6*8(sp)
    ld x7, 7*8(sp)
    ld x8, 8*8(sp)
    ld x9, 9*8(sp)
    ld x10, 10*8(sp)
    ld x11, 11*8(sp)
    ld x12, 12*8(sp)
    ld x13, 13*8(sp)
    ld x14, 14*8(sp)
    ld x15, 15*8(sp)
    ld x16, 16*8(sp)
    ld x17, 17*8(sp)
    ld x18, 18*8(sp)
    ld x19, 19*8(sp)
    ld x20, 20*8(sp)
    ld x21, 21*8(sp)
    ld x22, 22*8(sp)
    ld x23, 23*8(sp)
    ld x24, 24*8(sp)
    ld x25, 25*8(sp)
    ld x26, 26*8(sp)
    ld x27, 27*8(sp)
    ld x28, 28*8(sp)
    ld x29, 29*8(sp)
    ld x30, 30*8(sp)
    ld x31, 31*8(sp)
    addi sp, sp, 8*31
    sret

.text
.extern VS_EXCEPTION
.global virtual_supervisor_exception_handler 
.balign 256
virtual_supervisor_exception_handler:
    addi sp, sp, -8*31
    sd x0, 0*8(sp)
    sd x1, 1*8(sp)
    sd x2, 2*8(sp)
    sd x3, 3*8(sp)
    sd x4, 4*8(sp)
    sd x5, 5*8(sp)
    sd x6, 6*8(sp)
    sd x7, 7*8(sp)
    sd x8, 8*8(sp)
    sd x9, 9*8(sp)
    sd x10, 10*8(sp)
    sd x11, 11*8(sp)
    sd x12, 12*8(sp)
    sd x13, 13*8(sp)
    sd x14, 14*8(sp)
    sd x15, 15*8(sp)
    sd x16, 16*8(sp)
    sd x17, 17*8(sp)
    sd x18, 18*8(sp)
    sd x19, 19*8(sp)
    sd x20, 20*8(sp)
    sd x21, 21*8(sp)
    sd x22, 22*8(sp)
    sd x23, 23*8(sp)
    sd x24, 24*8(sp)
    sd x25, 25*8(sp)
    sd x26, 26*8(sp)
    sd x27, 27*8(sp)
    sd x28, 28*8(sp)
    sd x29, 29*8(sp)
    sd x30, 30*8(sp)
    sd x31, 31*8(sp)
    lb a0, VS_EXCEPTION
    mv a1, sp
    call exception_handler
    ld x0, 0*8(sp)
    ld x1, 1*8(sp)
    ld x2, 2*8(sp)
    ld x3, 3*8(sp)
    ld x4, 4*8(sp)
    ld x5, 5*8(sp)
    ld x6, 6*8(sp)
    ld x7, 7*8(sp)
    ld x8, 8*8(sp)
    ld x9, 9*8(sp)
    ld x10, 10*8(sp)
    ld x11, 11*8(sp)
    ld x12, 12*8(sp)
    ld x13, 13*8(sp)
    ld x14, 14*8(sp)
    ld x15, 15*8(sp)
    ld x16, 16*8(sp)
    ld x17, 17*8(sp)
    ld x18, 18*8(sp)
    ld x19, 19*8(sp)
    ld x20, 20*8(sp)
    ld x21, 21*8(sp)
    ld x22, 22*8(sp)
    ld x23, 23*8(sp)
    ld x24, 24*8(sp)
    ld x25, 25*8(sp)
    ld x26, 26*8(sp)
    ld x27, 27*8(sp)
    ld x28, 28*8(sp)
    ld x29, 29*8(sp)
    ld x30, 30*8(sp)
    ld x31, 31*8(sp)
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
    scause == E_STORE_AMO_GUEST_PAGE_FAULT || scause == E_LOAD_GUEST_PAGE_FAULT
}

fn is_instruction_abort(scause: usize) -> bool {
    scause == E_ILLEGAL_INSTRUCTION
        || scause == E_VIRTUAL_INSTRUCTION
        || scause == E_ENVIRONMENT_CALL_FROM_VS_MODE
}

#[no_mangle]
pub fn exception_handler(mode: u8, sp: usize) {
    if mode == M_EXCEPTION {
        if get_mcause() as usize == I_MACHINE_EXTERNAL {
            let hart = get_mhartid() as usize;
            if plic::get_plic_claim(hart) != plic::UART_IRQ {
                panic!();
            }
            let c = ns16550::ns16550_get_by_offset(ns16550::NS16500_RBR);
            ns16550::uart_fifo_push(c as u8);
            plic::set_plic_claim(hart, plic::UART_IRQ);
            // next instruction
            let mut sepc = get_sepc();
            sepc += 4;
            set_sepc(sepc);
            return;
        }
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

    let contexts = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32) };
    if is_data_abort(scause) {
        // data abort
        data_abort_handler(scause, contexts);
    } else if is_instruction_abort(scause) {
        // instruction abort
        instruction_abort_handler(scause, contexts);
    } else {
        println!("Exception from S-Mode has occured!");
        println!("[info] scause: {:#X}", get_scause());
        println!("[info] stval: {:#X}", get_stval());

        panic!();
    }
    // next instruction
    let mut sepc = get_sepc();
    sepc += 4;
    set_sepc(sepc);
}

fn write_access(virtual_address: usize, value: u64) {
    if (ns16550::NS16550_ADDR..=ns16550::NS16550_ADDR + 0x100).contains(&(virtual_address)) {
        ns16550::emulate_write_ns16550(virtual_address - ns16550::NS16550_ADDR, value as u32);
    } else if (virtio::VIRTIO_MMIO_DEFAULT_ADDRESS..=virtio::VIRTIO_MMIO_DEFAULT_ADDRESS + 0x1000).contains(&(virtual_address)) {
        virtio::emulate_write_virtio(virtual_address - virtio::VIRTIO_MMIO_DEFAULT_ADDRESS, value as u32);
    } else {
        println!("write access data abort");
        println!("[info] virtual address: {:#X}", virtual_address);
        let physical_address = paging::resolve_address_stage2(virtual_address).unwrap();
        println!("[info] physical address: {:#X}", physical_address);
        panic!();
    }
}

fn read_access(virtual_address: usize, dst_register_idx: usize, registers: &mut [u64]) {
    if (ns16550::NS16550_ADDR..=ns16550::NS16550_ADDR + 0x100).contains(&(virtual_address)) {
        registers[dst_register_idx] =
            ns16550::emulate_read_ns16550(virtual_address - ns16550::NS16550_ADDR).unwrap() as u64;
    } else if (virtio::VIRTIO_MMIO_DEFAULT_ADDRESS..=virtio::VIRTIO_MMIO_DEFAULT_ADDRESS + 0x1000).contains(&(virtual_address)) {
        registers[dst_register_idx] =
            virtio::emulate_read_virtio(virtual_address - virtio::VIRTIO_MMIO_DEFAULT_ADDRESS).unwrap() as u64;
    } else {
        println!("read access data abort");
        println!("[info] virtual address: {:#X}", virtual_address);
        let physical_address = paging::resolve_address_stage2(virtual_address).unwrap();
        println!("[info] physical address: {:#X}", physical_address);
        panic!();
    }
}

fn data_abort_handler(scause: usize, registers: &mut [u64]) {
    let mut instruction = instruction::Instruction::new(get_htinst() as u32);
    match scause {
        E_STORE_AMO_GUEST_PAGE_FAULT => {
            // write access
            let stval = get_stval() as usize;
            let register_idx = instruction.get_rs2();
            let value = registers[register_idx];
            write_access(stval, value);
        }
        E_LOAD_GUEST_PAGE_FAULT => {
            let stval = get_stval() as usize;
            let register_idx = instruction.get_rd();
            read_access(stval, register_idx, registers);
        }
        _ => {}
    };
}

fn instruction_abort_handler(scause: usize, registers: &mut [u64]) {
    match scause {
        E_ILLEGAL_INSTRUCTION => {
            println!("[info] E_ILLEGAL_INSTRUCTION: {:#x}", get_stval());
            println!("[info] hgatp: {:#x}", get_hgatp());
            println!("[info] virtual address: {:#x}", get_sepc());
            println!(
                "[info] physical address: {:#x}",
                paging::resolve_address_stage2(get_sepc() as usize).unwrap()
            );
            panic!();
        }
        E_VIRTUAL_INSTRUCTION => {
            let mut instruction = instruction::Instruction::new(get_stval() as u32);
            if instruction.is_csrrs_instruction() {
                let csr = instruction.get_funct12();
                if csr == CSR_TIME_ADDRESS {
                    let register_number = instruction.get_rd();
                    registers[register_number] = get_time();
                }
            } else {
                println!("[info] VIRTUAL INSTRUCTION: {:#x}", get_stval());
                println!("[info] virtual address: {:#x}", get_sepc());
                println!(
                    "[info] physical address: {:#x}",
                    paging::resolve_address_stage2(get_sepc() as usize).unwrap()
                );
                panic!();
            }
        }
        E_ENVIRONMENT_CALL_FROM_VS_MODE => {
            // TODO: 割り込み時のコンテキストをスタック上ではなくVM構造体に直接保存
            let a6 = registers[REGISTER_A6];
            let a7 = registers[REGISTER_A7];
            let mut sbi_ret = sbi::Sbiret { error: 0, value: 0 };
            sbi::virtual_sbi(&mut sbi_ret, a7, a6);
            registers[REGISTER_A0] = sbi_ret.error; // a0
            registers[REGISTER_A1] = sbi_ret.value; // a1;
        }
        _ => {
            println!("Exception from S-mode has occured!");
            println!("[info] virtual address: {:#X}", get_sepc());
            let physical_address = paging::resolve_address_stage2(get_sepc() as usize).unwrap();
            println!("[info] physical address: {:#X}", physical_address);
            panic!();
        }
    };
}
