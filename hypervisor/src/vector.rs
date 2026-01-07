use crate::CURRENT_VMID;
use crate::VIRTUAL_MACHINES;
use crate::mmio::ns16550;
use crate::paging;
use crate::plic;
use crate::println;
use crate::sbi;
use crate::vm::{Csr, VM};
use alloc::vec::Vec;
use arch::riscv::cpu::csr_address::CSR_HGATP_ADDRESS;
use arch::riscv::cpu::csr_address::CSR_TIME_ADDRESS;
use arch::riscv::instruction::CsrAccessInstructionType;
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
    let mut locked_vm = match VIRTUAL_MACHINES.try_lock() {
        Some(vms) => vms,
        None => panic!("VIRTUAL_MACHINES is locked."),
    };
    let mut locked_current_vmid = CURRENT_VMID.lock();

    let scause = get_scause() as usize;
    let sp = get_sscratch() as usize;

    #[cfg(feature = "nested_support")]
    match locked_vm[*locked_current_vmid].parent_vmid {
        Some(parent_vmid) => {
            // Switch Hypervisor Context from L1 to L0
            load_hypervisor_context(*(HOST_HYPERVISOR_CSR.lock()));

            // Change Current VMID from L2 VM to L1 VM
            switch_vm_context(parent_vmid, &mut locked_current_vmid, &mut locked_vm);

            println!("↓L2 VM ↑L1 VMM");
            let csr = locked_vm[parent_vmid].vcsr;

            drop(locked_current_vmid);
            drop(locked_vm);

            if is_data_abort(scause) {
                // Assert Page Fault to L1 Hypervisor
                assert_l1_hypervisor(get_scause(), get_sepc(), get_stval(), csr.stvec);
                // TODO: Consider that L1 changed page table.
            } else if is_instruction_abort(scause) {
                assert_l1_hypervisor(get_scause(), get_sepc(), get_stval(), csr.stvec);
            }

            // don't return to here.
            panic!();
        }
        None => {
            // L1 VM
        }
    }

    let contexts = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32) };
    if is_data_abort(scause) {
        // data abort
        let vm = &mut locked_vm[*locked_current_vmid];
        let mmio_list = &mut vm.mmio;
        data_abort_handler(scause, contexts, mmio_list);
    } else if is_instruction_abort(scause) {
        // instruction abort
        instruction_abort_handler(scause, contexts, locked_current_vmid, locked_vm);
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

fn data_abort_handler(scause: usize, registers: &mut [u64], mmios: &mut Vec<MmioEntry>) {
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
    mut mutex_vms: MutexGuard<'_, Vec<VM>>,
) {
    let current_vmid = *mutex_vmid;
    let vms = &mut *mutex_vms;
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

            if let Some(access_type) = instruction.is_csr_access() {
                let csr_address = instruction.get_funct12();
                // dst register number
                let rd = instruction.get_rd();
                // src register number
                let rs1 = instruction.get_rs1();

                #[cfg(feature = "nested_support")]
                if csr_address::is_hypervisor_csr(csr_address) {
                    // Access to a Hypervisor CSR from an L1 implies that
                    // a hypervisor is running within the L1 VM.
                    let mut l1_hypervisor = match vms[current_vmid].hypervisor {
                        Some(context) => context,
                        None => {
                            let vmid = create_l2_vm(current_vmid, vms);
                            HypervisorContext {
                                csr: HypervisorCsr::new(),
                                vmid,
                            }
                        }
                    };
                    let write_value = match access_type {
                        CsrAccessInstructionType::CSRRW => registers[rs1],
                        CsrAccessInstructionType::CSRRS => {
                            let reg_value = registers[rs1];
                            let csr_value = l1_hypervisor.csr.get_csr(csr_address);
                            csr_value | reg_value
                        }
                    };

                    emulate_csr(&mut l1_hypervisor, csr_address, rd, write_value, registers);

                    if csr_address == CSR_HGATP_ADDRESS {
                        let l2_vmid = l1_hypervisor.vmid;
                        match paging::shadow_map_address_stage2(
                            true,
                            true,
                            true,
                            l1_hypervisor.csr.hgatp,
                        ) {
                            Ok(table_address) => {
                                vms[l2_vmid].page_table_address = table_address;
                            }
                            Err(err) => {
                                println!(
                                    "Error: Failed to create shadow page table for L2 VM: {}",
                                    err
                                );
                                return;
                            }
                        }
                    }

                    vms[current_vmid].hypervisor = Some(l1_hypervisor);

                    return;
                }

                if csr_address == CSR_TIME_ADDRESS {
                    // Read Only
                    registers[rd] = get_time();

                    return;
                }

                panic!("Unsupported CSR address: {:#x}", csr_address);
            }
            #[cfg(feature = "nested_support")]
            if instruction.is_sret() {
                println!("↓L1 VM ↑L2 VM");
                // L1 Hypervisor trying to context switching to L2 VM
                let l1_hypervisor = match vms[current_vmid].hypervisor {
                    Some(hypervisor) => hypervisor,
                    None => {
                        let vmid = create_l2_vm(current_vmid, vms);
                        HypervisorContext {
                            csr: HypervisorCsr::new(),
                            vmid,
                        }
                    }
                };
                let l2_vmid = l1_hypervisor.vmid;
                if l1_hypervisor.csr.hstatus as usize & HSTATUS_SPV == 0 {
                    // L1 Hypervisor must be set this bit
                    panic!("Invalid SRET");
                }

                // Switch Hypervisor Context from L0 to L1
                store_l0_hypervisor_context();
                load_hypervisor_context(l1_hypervisor.csr);

                // Switch Page Table from L1 VM to L2 VM
                let table_address = vms[l1_hypervisor.vmid].page_table_address;
                let mut hgatp = match paging::DEFAULT_TABLE_LEVEL {
                    3 => 0b1000 << 60,
                    4 => 0b1001 << 60,
                    5 => 0b1010 << 60,
                    _ => unreachable!(),
                };
                hgatp |= (table_address >> 12) & SATP_PPN_MASK;
                set_hgatp(hgatp as u64);

                // Change Current VMID from L1 VM to L2 VM
                switch_vm_context(l2_vmid, &mut mutex_vmid, &mut mutex_vms);

                // Set L2 VM entry point
                set_sepc(get_vsepc());

                println!("L2 VM entry point: {:#x}", get_vsepc() as usize);

                drop(mutex_vmid);
                drop(mutex_vms);

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

fn switch_vm_context(
    vmid: usize,
    mutex_vmid: &mut MutexGuard<'_, usize>,
    mutex_vms: &mut MutexGuard<'_, Vec<VM>>,
) {
    let current_vm = &mut (*mutex_vms)[**mutex_vmid];

    let csr = Csr {
        stvec: get_vstvec(),
        sepc: get_vsepc(),
        sstatus: get_vsstatus(),
        scause: get_vscause(),
        stval: get_vstval(),
        satp: get_vsatp(),
    };
    current_vm.vcsr = csr;

    **mutex_vmid = vmid;
}

#[cfg(feature = "nested_support")]
fn assert_l1_hypervisor(vscause: u64, vsepc: u64, vstval: u64, sepc: u64) {
    set_vscause(vscause);
    set_vsepc(vsepc);
    set_vstval(vstval);
    set_sepc(sepc);

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
    set_henvcfg(csr.henvcfg);
    set_hgatp(csr.hgatp);
}
