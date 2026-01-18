use crate::paging;
use crate::println;
use crate::vm::VM;
use crate::vm::switch_vm_context;
use alloc::vec::Vec;
use arch::riscv::cpu::csr_address::CSR_HGATP_ADDRESS;
use arch::riscv::cpu::*;
use arch::riscv::instruction::CsrAccessInstructionType;
use spin::MutexGuard;
#[cfg(feature = "nested_acceleration")]
use {
    crate::BUFFER_COUNT,
    crate::shmem_handle::*,
    crate::timer::{TIMER_FRQ, disable_timer_intr, start_timer},
    crate::vector::{E_STORE_AMO_GUEST_PAGE_FAULT, I_VIRTUAL_SUPERVISOR_SOFTWARE},
    arch::riscv::instruction,
};
use {
    crate::HOST_HYPERVISOR_CSR, crate::emulate_csr::HypervisorCsr, crate::emulate_csr::emulate_csr,
    crate::vm::HypervisorContext,
};

#[cfg(feature = "nested_acceleration")]
const TARGET_ADDRESS: usize = 0x10000000;
#[cfg(feature = "nested_acceleration")]
const FLUSH_INTERVAL: usize = 5;

pub fn assert_l1_hypervisor(sp: usize, vscause: u64, vsepc: u64, vstval: u64, sepc: u64) -> ! {
    set_vscause(vscause);
    set_vsepc(vsepc);
    set_vstval(vstval);
    set_sepc(sepc);

    unsafe extern "C" {
        fn vm_entry(sp: usize);
    }
    unsafe {
        vm_entry(sp);
    }
    // don't return to here
    unreachable!()
}

pub fn store_l0_hypervisor_context() {
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

pub fn load_hypervisor_context(csr: HypervisorCsr) {
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

pub fn create_l2_vm(parent_vmid: usize, vms: &mut Vec<VM>) -> usize {
    let new_vmid = vms.len();
    let l2_vm = VM::new(new_vmid, 0, 0, 0, 0, 0, 0, Vec::new(), Some(parent_vmid));
    vms.push(l2_vm);

    new_vmid
}

pub fn hypervisor_csr_access(
    registers: &mut [u64],
    current_vmid: usize,
    vms: &mut Vec<VM>,
    csr_address: usize,
    access_type: CsrAccessInstructionType,
    src: usize,
    dst: usize,
) {
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
        CsrAccessInstructionType::CSRRW => registers[src],
        CsrAccessInstructionType::CSRRS => {
            let reg_value = registers[src];
            let csr_value = l1_hypervisor.csr.get_csr(csr_address);
            csr_value | reg_value
        }
    };

    emulate_csr(&mut l1_hypervisor, csr_address, dst, write_value, registers);

    if csr_address == CSR_HGATP_ADDRESS {
        let l2_vmid = l1_hypervisor.vmid;
        match paging::shadow_map_address_stage2(true, true, true, l1_hypervisor.csr.hgatp) {
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
}

#[cfg(feature = "nested_acceleration")]
pub fn timer_flush_to_l1(
    sp: usize,
    mut mutex_vmid: MutexGuard<'_, usize>,
    mut mutex_vms: MutexGuard<'_, Vec<VM>>,
) {
    if let Some(parent_vmid) = mutex_vms[*mutex_vmid].parent_vmid {
        disable_timer_intr();
        load_hypervisor_context(*(HOST_HYPERVISOR_CSR.lock()));
        switch_vm_context(parent_vmid, &mut mutex_vmid, &mut mutex_vms);
    } else {
        // print!(".");
        start_timer((FLUSH_INTERVAL * TIMER_FRQ) as u64);
        return;
    }
    let csr = mutex_vms[*mutex_vmid].vcsr;

    drop(mutex_vmid);
    drop(mutex_vms);

    assert_l1_hypervisor(
        sp,
        I_VIRTUAL_SUPERVISOR_SOFTWARE as u64,
        get_sepc(),
        TARGET_ADDRESS as u64,
        csr.stvec,
    );
}

pub fn reflect_to_l1(
    sp: usize,
    mut mutex_vmid: MutexGuard<'_, usize>,
    mut mutex_vms: MutexGuard<'_, Vec<VM>>,
    parent_vmid: usize,
) {
    // return to L2
    #[cfg(feature = "nested_acceleration")]
    if get_scause() as usize == E_STORE_AMO_GUEST_PAGE_FAULT
        && get_stval() as usize == TARGET_ADDRESS
    {
        let contexts = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u64, 32) };
        let instruction = instruction::Instruction::new(get_htinst() as u32);
        let rs2 = instruction.get_rs2();
        let byte = contexts[rs2] as u8;

        with_shm_ring_mut(|r| {
            r.push(&[byte]);
        });

        let mut cnt = BUFFER_COUNT.lock();
        *cnt += 1;
        let do_flush = *cnt > 16 || byte == b'\n';
        if do_flush {
            disable_timer_intr();
            *cnt = 0;
        }
        drop(cnt);

        let inst_len = if instruction.is_compression_instruction() {
            2
        } else {
            4
        };

        if do_flush {
            load_hypervisor_context(*(HOST_HYPERVISOR_CSR.lock()));
            switch_vm_context(parent_vmid, &mut mutex_vmid, &mut mutex_vms);

            let csr = mutex_vms[parent_vmid].vcsr;

            drop(mutex_vmid);
            drop(mutex_vms);

            assert_l1_hypervisor(
                sp,
                I_VIRTUAL_SUPERVISOR_SOFTWARE as u64,
                get_sepc() + inst_len,
                TARGET_ADDRESS as u64,
                csr.stvec,
            );
        }

        start_timer((FLUSH_INTERVAL * TIMER_FRQ) as u64);

        set_sepc(get_sepc() + inst_len);

        return;
    }

    // assert to L1
    load_hypervisor_context(*(HOST_HYPERVISOR_CSR.lock()));
    switch_vm_context(parent_vmid, &mut mutex_vmid, &mut mutex_vms);

    // println!("↓L2 VM ↑L1 VMM");
    if let Some(hypervisor) = mutex_vms[parent_vmid].hypervisor.as_mut() {
        hypervisor.csr.htinst = get_htinst();
        hypervisor.csr.htval = get_htval();
    } else {
        panic!("No L1 Hypervisor.");
    }

    let csr = mutex_vms[parent_vmid].vcsr;

    drop(mutex_vmid);
    drop(mutex_vms);

    assert_l1_hypervisor(sp, get_scause(), get_sepc(), get_stval(), csr.stvec);
    // don't return to here.
}

pub fn l1_to_l2(
    sp: usize,
    mut mutex_vmid: MutexGuard<'_, usize>,
    mut mutex_vms: MutexGuard<'_, Vec<VM>>,
) {
    // println!("↓L1 VM ↑L2 VM");
    let current_vmid = *mutex_vmid;
    // L1 Hypervisor trying to context switching to L2 VM
    let l1_hypervisor = match mutex_vms[current_vmid].hypervisor {
        Some(hypervisor) => hypervisor,
        None => {
            let vmid = create_l2_vm(current_vmid, &mut mutex_vms);
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
    let table_address = mutex_vms[l1_hypervisor.vmid].page_table_address;
    let mut hgatp = match paging::DEFAULT_TABLE_LEVEL {
        3 => 0b1000 << 60,
        4 => 0b1001 << 60,
        5 => 0b1010 << 60,
        _ => unreachable!(),
    };
    hgatp |= (table_address >> 12) & SATP_PPN_MASK;
    set_hgatp(hgatp as u64);

    // Set L2 VM entry point
    set_sepc(get_vsepc());

    // println!("L2 VM entry point: {:#x}", get_vsepc() as usize);

    // Change Current VMID from L1 VM to L2 VM
    switch_vm_context(l2_vmid, &mut mutex_vmid, &mut mutex_vms);

    drop(mutex_vmid);
    drop(mutex_vms);

    unsafe extern "C" {
        fn vm_entry(sp: usize);
    }
    unsafe {
        vm_entry(sp);
    }
    // don't return to here
}
