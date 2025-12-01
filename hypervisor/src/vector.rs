use crate::CURRENT_VMID;
use crate::VIRTUAL_MACHINES;
use crate::mmio::ns16550;
use crate::paging;
use crate::plic;
use crate::println;
use crate::sbi;
use crate::vm::VM;
use alloc::vec::Vec;
use arch::riscv::cpu::csr_address::CSR_TIME_ADDRESS;
use arch::riscv::{cpu::*, instruction, instruction::Instruction};
use core::arch::global_asm;
use mmio_core::MmioEntry;
use spin::MutexGuard;
#[cfg(feature = "nested_support")]
use {
    crate::HOST_HYPERVISOR_CSR, crate::emulate_csr::HypervisorCsr, crate::emulate_csr::emulate_csr,
    crate::vm::HypervisorContext, crate::vm::create_l2_vm,
};

pub const E_ILLEGAL_INSTRUCTION: usize = 2;
pub const E_INSTRUCTION_GUEST_PAGE_FAULT: usize = 20;
pub const E_LOAD_GUEST_PAGE_FAULT: usize = 21;
pub const E_VIRTUAL_INSTRUCTION: usize = 22;
pub const E_STORE_AMO_GUEST_PAGE_FAULT: usize = 23;
pub const E_ENVIRONMENT_CALL_FROM_VS_MODE: usize = 10;

pub const INTERRUPT_ID: usize = 1 << (MXLEN - 1);
pub const I_MACHINE_EXTERNAL: usize = 11 | INTERRUPT_ID;

global_asm!(include_str!("./trap.S"));

pub fn setup_vector() {
    unsafe extern "C" {
        static machine_vector_table: *const u8;
        static supervisor_vector_table: *const u8;
    }
    unsafe { set_mtvec((&machine_vector_table as *const _ as usize) as u64) }
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
pub fn machine_handler() {
    let mcause = get_mcause();
    match mcause as usize {
        I_MACHINE_EXTERNAL => {
            let hart = get_mhartid() as usize;
            let claim = plic::get_plic_claim(hart);
            match claim {
                plic::UART_IRQ => {
                    let c = ns16550::ns16550_get_by_offset(ns16550::NS16500_RBR);
                    ns16550::uart_fifo_push(c as u8);
                    plic::set_plic_claim(hart, plic::UART_IRQ);
                }
                _ => {
                    println!("claim: {}", claim);
                    panic!();
                }
            }
        }
        _ => {
            println!("Exception from M-Mode has occured!");
            println!("[info] mcause: {:#X}", mcause);
            println!("[info] mtval: {:#X}", get_mtval());

            if mcause as usize == E_INSTRUCTION_GUEST_PAGE_FAULT {
                println!("[info] mtinst: {:#X}", get_mtinst());
            }
            panic!();
        }
    }
    // Since a trap into M-Mode is an asynchronous exception,
    // the mepc is not incremented.
}

#[unsafe(no_mangle)]
pub fn exception_handler() {
    let mut locked_vm = VIRTUAL_MACHINES.lock();
    let locked_current_vmid = CURRENT_VMID.lock();

    let scause = get_scause() as usize;
    let sp = get_sscratch() as usize;

    if cfg!(feature = "nested_support") {
        let current_vm = &locked_vm[*locked_current_vmid];
        match current_vm.parent_vmid {
            Some(parent_vmid) => {
                // L2 VM
                if is_data_abort(scause) {
                    // Assert Page Fault to L1 Hypervisor
                    assert_l1_hypervisor(
                        get_vscause(),
                        get_vsepc(),
                        get_vstval(),
                    );
                    // TODO: Consider that L1 changed page table.
                } else if is_instruction_abort(scause) {
                    assert_l1_hypervisor(
                        get_vscause(),
                        get_vsepc(),
                        get_vstval(),
                    );
                }

                // don't return to here.
            }
            None => {
                // L1 VM
            }
        }
    }

    let contexts = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32) };
    if is_data_abort(scause) {
        // data abort
        let vm = &locked_vm[*locked_current_vmid];
        let mmio_list = &vm.mmio;
        data_abort_handler(scause, contexts, mmio_list);
    } else if is_instruction_abort(scause) {
        // instruction abort
        instruction_abort_handler(scause, contexts, locked_current_vmid, &mut *locked_vm);
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

fn write_access(virtual_address: usize, value: u64, mmios: &Vec<MmioEntry>) {
    for mmio in mmios.iter() {
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

fn data_abort_handler(scause: usize, registers: &mut [u64], mmios: &Vec<MmioEntry>) {
    let instruction = instruction::Instruction::new(get_htinst() as u32);
    match scause {
        E_STORE_AMO_GUEST_PAGE_FAULT => {
            // write access
            let stval = get_stval() as usize;
            let register_idx = instruction.get_rs2();
            let value = registers[register_idx];
            write_access(stval, value, mmios);
        }
        E_LOAD_GUEST_PAGE_FAULT => {
            let stval = get_stval() as usize;
            let register_idx = instruction.get_rd();
            read_access(stval, register_idx, registers, mmios);
        }
        _ => {}
    };
}

fn instruction_abort_handler(
    scause: usize,
    registers: &mut [u64],
    mut mutex_vmid: MutexGuard<'_, usize>,
    vms: &mut Vec<VM>,
) {
    let current_vmid = *mutex_vmid;
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

            if instruction.is_csrrw_instruction() {
                let vm = &mut vms[current_vmid];
                let csr_address = instruction.get_funct12();
                #[cfg(feature = "nested_support")]
                if csr_address::is_hypervisor_csr(csr_address) {
                    // Access to a Hypervisor CSR from an L1 implies that
                    // a hypervisor is running within the L1 VM.
                    let mut l1_hypervisor = match vm.hypervisor {
                        Some(context) => context,
                        None => HypervisorContext {
                            csr: HypervisorCsr::new(),
                            vmid: None,
                        },
                    };
                    println!("create L1 Hypervisor");
                    let rd = instruction.get_rd();
                    let rs1 = instruction.get_rs1();
                    let write_value = registers[rs1];
                    emulate_csr(&mut l1_hypervisor, csr_address, rd, write_value, registers);
                    vm.hypervisor = Some(l1_hypervisor);

                    return;
                }

                println!("CSRRW: {:#x}", csr_address);
                panic!();
            }
            if instruction.is_csrrs_instruction() {
                let vm = &mut vms[current_vmid];
                let csr_address = instruction.get_funct12();
                #[cfg(feature = "nested_support")]
                if csr_address::is_hypervisor_csr(csr_address) {
                    // Access to a Hypervisor CSR from an L1 implies that
                    // a hypervisor is running within the L1 VM.
                    let mut l1_hypervisor = match vm.hypervisor {
                        Some(context) => context,
                        None => {
                            // Create L2 VM
                            HypervisorContext {
                                csr: HypervisorCsr::new(),
                                vmid: None,
                            }
                        }
                    };
                    let virtual_csr = l1_hypervisor.csr;
                    let rd = instruction.get_rd();
                    let rs1 = instruction.get_rs1();
                    let reg_value = registers[rs1];
                    let csr_value = virtual_csr.get_csr(csr_address);
                    let write_value = csr_value | reg_value;
                    emulate_csr(&mut l1_hypervisor, csr_address, rd, write_value, registers);
                    vm.hypervisor = Some(l1_hypervisor);

                    return;
                }
                if csr_address == CSR_TIME_ADDRESS {
                    // Read Only
                    let rd = instruction.get_rd();
                    registers[rd] = get_time();

                    return;
                }

                println!("CSRRS: {:#x}", csr_address);
                panic!();
            }
            #[cfg(feature = "nested_support")]
            if instruction.is_sret() {
                // L1 Hypervisor trying to context switching to L2 VM
                let l1_hypervisor = match vms[current_vmid].hypervisor {
                    Some(hypervisor) => hypervisor,
                    None => HypervisorContext::new(),
                };
                let l2_vmid = match l1_hypervisor.vmid {
                    Some(vmid) => vmid,
                    None => {
                        // Create L2 VM
                        println!("create L2 VM");
                        let vmid = create_l2_vm(current_vmid, vms);
                        vms[current_vmid].hypervisor.unwrap().vmid = Some(vmid);
                        vmid
                    }
                };
                if l1_hypervisor.csr.hstatus as usize & HSTATUS_SPV == 0 {
                    // L1 Hypervisor must be set this bit
                    panic!("Invalid SRET");
                }

                // Switch Hypervisor Context from L0 to L1
                store_l0_hypervisor_context();
                load_hypervisor_context(l1_hypervisor.csr);

                // Change Current VMID from L1 VM to L2 VM
                *mutex_vmid = l2_vmid;

                // Set L2 VM entry point
                set_sepc(get_vsepc());

                unsafe extern "C" {
                    fn vm_entry();
                }
                unsafe {
                    vm_entry();
                }
                // don't return to here
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
            let sbi_ret = sbi::virtual_sbi(a7, a6, a0, a1, a2, a3, a4, a5);
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

#[cfg(feature = "nested_support")]
fn assert_l1_hypervisor(
    scause: u64,
    sepc: u64,
    stval: u64,
) {
    set_scause(scause);
    set_sepc(sepc);
    set_stval(stval);

    unsafe extern "C" {
        fn vm_entry();
    }
    unsafe {
        vm_entry();
    }
    // don't return to here
}

#[cfg(feature = "nested_support")]
fn store_l0_hypervisor_context() {
    let mut locked_csr = HOST_HYPERVISOR_CSR.lock();
    let host_csr = HypervisorCsr {
        hstatus: get_hstatus(),
        hedeleg: get_hedeleg(),
        hideleg: get_hideleg(),
        hie: get_hie(),
        hcounteren: get_hcounteren(),
        hgeie: get_hgeie(),
        htval: get_htval(),
        hip: get_hip(),
        hvip: get_hvip(),
        htinst: get_htinst(),
        hgeip: get_hgeip(),
        henvcfg: get_henvcfg(),
        hgatp: get_hgatp(),
    };
    *locked_csr = host_csr;
}

#[cfg(feature = "nested_support")]
fn load_hypervisor_context(csr: HypervisorCsr) {
    set_hstatus(csr.hstatus);
    set_hedeleg(csr.hedeleg);
    set_hideleg(csr.hideleg);
    set_hie(csr.hie);
    set_hcounteren(csr.hcounteren);
    set_hgeie(csr.hgeie);
    set_htval(csr.htval);
    set_hip(csr.hip);
    set_hvip(csr.hvip);
    set_htinst(csr.htinst);
    set_hgeip(csr.hgeip);
    set_henvcfg(csr.henvcfg);
    set_hgatp(csr.hgatp);
}
