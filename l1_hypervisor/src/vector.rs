use crate::CURRENT_VMID;
use crate::VIRTUAL_MACHINES;
use crate::paging;
use crate::println;
use crate::sbi;
use crate::with_shm_ring;
use alloc::vec::Vec;

use arch::riscv::cpu::csr_address::CSR_TIME_ADDRESS;
use arch::riscv::{cpu::*, instruction, instruction::Instruction};
use core::arch::global_asm;
use mmio_core::MmioEntry;

pub const E_ILLEGAL_INSTRUCTION: usize = 2;
pub const E_LOAD_GUEST_PAGE_FAULT: usize = 21;
pub const E_VIRTUAL_INSTRUCTION: usize = 22;
pub const E_STORE_AMO_GUEST_PAGE_FAULT: usize = 23;
pub const E_ENVIRONMENT_CALL_FROM_VS_MODE: usize = 10;

pub const INTERRUPT_ID: usize = 1 << (MXLEN - 1);
pub const I_VIRTUAL_SUPERVISOR_SOFTWARE: usize = 2 | INTERRUPT_ID;

global_asm!(include_str!("./trap.S"));

pub fn enable_intr() {
    set_sstatus(get_sstatus() | SSTATUS_SIE);
}

pub fn disable_intr() {
    set_sstatus(get_sstatus() & !SSTATUS_SIE);
}

struct IrqSafe;

impl IrqSafe {
    fn new() -> Self {
        disable_intr();
        Self
    }
}

impl Drop for IrqSafe {
    fn drop(&mut self) {
        enable_intr();
    }
}

pub fn setup_vector() {
    unsafe extern "C" {
        static supervisor_vector_table: *const u8;
    }
    unsafe { set_stvec((&supervisor_vector_table as *const _ as usize) as u64) }
}

fn is_data_abort(scause: usize) -> bool {
    scause == E_STORE_AMO_GUEST_PAGE_FAULT || scause == E_LOAD_GUEST_PAGE_FAULT
}

fn is_instruction_abort(scause: usize) -> bool {
    scause == E_ILLEGAL_INSTRUCTION
        || scause == E_VIRTUAL_INSTRUCTION
        || scause == E_ENVIRONMENT_CALL_FROM_VS_MODE
}

#[unsafe(no_mangle)]
pub fn exception_handler(sp: usize) {
    let scause = get_scause() as usize;

    let contexts = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32) };
    if is_data_abort(scause) {
        data_abort_handler(scause, contexts);
    } else if is_instruction_abort(scause) {
        instruction_abort_handler(scause, contexts);
    } else if scause == I_VIRTUAL_SUPERVISOR_SOFTWARE {
        let buf = with_shm_ring(|r| {
            let mut buf = [0u8; 256];
            let n = r.pop(&mut buf);
            buf[0..n].to_vec()
        });
        let _ = IrqSafe::new();
        let mut vms = VIRTUAL_MACHINES.write();
        let current_vmid = CURRENT_VMID.write();
        let vm = &mut vms[*current_vmid];
        let mmio_list = &mut vm.mmio;
        let stval = get_stval() as usize;
        for byte in buf {
            write_access(stval, byte as u64, mmio_list);
        }
        return;
    } else {
        println!("Exception from S-Mode has occured!");
        println!("[info] scause: {:#X}", get_scause());
        println!("[info] stval: {:#X}", get_stval());

        panic!();
    }

    let mut instruction = Instruction::new(get_htinst() as u32);

    // next instruction
    let mut sepc = get_sepc();
    let instruction_size = if instruction.is_valid_instruction() {
        if instruction.is_compression_instruction() {
            2
        } else {
            4
        }
    } else {
        4
    };
    sepc += instruction_size;
    set_sepc(sepc);
}

fn write_access(virtual_address: usize, value: u64, mmios: &mut Vec<MmioEntry>) {
    for mmio in mmios.iter_mut() {
        if (mmio.address..=mmio.address + mmio.size).contains(&virtual_address) {
            let offset = virtual_address - mmio.address;
            mmio.handler.write(offset, value as usize);
            return;
        }
    }
    println!("write access data abort");
    println!("[info] virtual address: {:#X}", virtual_address);
    panic!();
}

fn read_access(
    virtual_address: usize,
    dst_register_idx: usize,
    registers: &mut [u64],
    mmios: &Vec<MmioEntry>,
) {
    for mmio in mmios.iter() {
        if (mmio.address..=mmio.address + mmio.size).contains(&virtual_address) {
            let offset = virtual_address - mmio.address;
            registers[dst_register_idx] = mmio.handler.read(offset) as u64;
            return;
        }
    }
    println!("read access data abort");
    println!("[info] virtual address: {:#X}", virtual_address);
    panic!();
}

fn data_abort_handler(scause: usize, registers: &mut [u64]) {
    let instruction = instruction::Instruction::new(get_htinst() as u32);
    let vmid = CURRENT_VMID.read();
    match scause {
        E_STORE_AMO_GUEST_PAGE_FAULT => {
            let _ = IrqSafe::new();

            let mut vms = VIRTUAL_MACHINES.write();
            let vm = &mut vms[*vmid];
            let mmios = &mut vm.mmio;

            let stval = get_stval() as usize;
            let register_idx = instruction.get_rs2();
            let value = registers[register_idx];
            write_access(stval, value, mmios);
        }
        E_LOAD_GUEST_PAGE_FAULT => {
            let vms = VIRTUAL_MACHINES.read();
            let vm = &vms[*vmid];
            let mmios = &vm.mmio;

            let stval = get_stval() as usize;
            let register_idx = instruction.get_rd();
            read_access(stval, register_idx, registers, mmios);
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
            let instruction = instruction::Instruction::new(get_stval() as u32);

            if let Some(_) = instruction.is_csr_access() {
                let csr_address = instruction.get_funct12();
                // dst register number
                let rd = instruction.get_rd();
                // src register number
                // let rs1 = instruction.get_rs1();

                if csr_address == CSR_TIME_ADDRESS {
                    // Read Only
                    registers[rd] = get_time();

                    return;
                }

                panic!("Unsupported CSR address: {:#x}", csr_address);
            }

            println!("[info] VIRTUAL INSTRUCTION: {:#x}", get_stval());
            println!("[info] virtual address: {:#x}", get_sepc());
            println!(
                "[info] physical address: {:#x}",
                paging::resolve_address_stage2(get_sepc() as usize).unwrap()
            );
            panic!();
        }
        E_ENVIRONMENT_CALL_FROM_VS_MODE => {
            // TODO: 割り込み時のコンテキストをスタック上ではなくVM構造体に直接保存
            let a0 = registers[REGISTER_A0] as usize;
            let a1 = registers[REGISTER_A1] as usize;
            let a2 = registers[REGISTER_A2] as usize;
            let a3 = registers[REGISTER_A3] as usize;
            let a4 = registers[REGISTER_A4] as usize;
            let a5 = registers[REGISTER_A5] as usize;
            let a6 = registers[REGISTER_A6] as usize;
            let a7 = registers[REGISTER_A7] as usize;
            let sbi_ret = sbi::sbi_ecall(a7, a6, a0, a1, a2, a3, a4, a5);
            registers[REGISTER_A0] = sbi_ret.error as u64; // a0
            registers[REGISTER_A1] = sbi_ret.value as u64; // a1;
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
