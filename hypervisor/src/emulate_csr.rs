use crate::paging::shadow_map_address_stage2;
use crate::println;
use crate::vm::HypervisorContext;
use arch::riscv::cpu::csr_address::*;
use arch::riscv::cpu::*;

#[derive(Clone, Copy, Debug)]
pub struct HypervisorCsr {
    pub hstatus: u64,
    pub hedeleg: u64,
    pub hideleg: u64,
    pub hie: u64,
    pub hcounteren: u64,
    pub hgeie: u64,
    pub htval: u64,
    pub hip: u64,
    pub hvip: u64,
    pub htinst: u64,
    pub hgeip: u64,
    pub henvcfg: u64,
    pub hgatp: u64,
}

impl Default for HypervisorCsr {
    fn default() -> Self {
        Self::new()
    }
}

impl HypervisorCsr {
    pub const fn new() -> Self {
        HypervisorCsr {
            hstatus: 0,
            hedeleg: 0,
            hideleg: 0,
            hie: 0,
            hcounteren: 0,
            hgeie: 0,
            htval: 0,
            hip: 0,
            hvip: 0,
            htinst: 0,
            hgeip: 0,
            henvcfg: 0,
            hgatp: 0,
        }
    }

    pub fn get_csr(&self, csr_address: usize) -> u64 {
        match csr_address {
            CSR_TIME_ADDRESS => get_time(),
            CSR_HSTATUS_ADDRESS => self.hstatus,
            CSR_HEDELEG_ADDRESS => self.hedeleg,
            CSR_HIDELEG_ADDRESS => self.hideleg,
            CSR_HIE_ADDRESS => self.hie,
            CSR_HCOUNTEREN_ADDRESS => self.hcounteren,
            CSR_HGEIE_ADDRESS => self.hgeie,
            CSR_HTVAL_ADDRESS => self.htval,
            CSR_HIP_ADDRESS => self.hip,
            CSR_HVIP_ADDRESS => self.hvip,
            CSR_HTINST_ADDRESS => self.htinst,
            CSR_HGEIP_ADDRESS => self.hgeip,
            CSR_HENVCFG_ADDRESS => self.henvcfg,
            CSR_HGATP_ADDRESS => self.hgatp,
            _ => {
                println!("This csr number isn't supported: {:#x}", csr_address);
                panic!();
            }
        }
    }

    pub fn set_csr(&mut self, csr_address: usize, value: u64) {
        match csr_address {
            CSR_TIME_ADDRESS => {
                panic!("Attempted to write to read-only CSR_TIME_ADDRESS register");
            }
            CSR_HSTATUS_ADDRESS => {
                self.hstatus = value;
            }
            CSR_HEDELEG_ADDRESS => {
                self.hedeleg = value;
            }
            CSR_HIDELEG_ADDRESS => {
                self.hideleg = value;
            }
            CSR_HIE_ADDRESS => {
                self.hie = value;
            }
            CSR_HCOUNTEREN_ADDRESS => {
                self.hcounteren = value;
            }
            CSR_HGEIE_ADDRESS => {
                self.hgeie = value;
            }
            CSR_HTVAL_ADDRESS => {
                self.htval = value;
            }
            CSR_HIP_ADDRESS => {
                self.hip = value;
            }
            CSR_HVIP_ADDRESS => {
                self.hvip = value;
            }
            CSR_HTINST_ADDRESS => {
                self.htinst = value;
            }
            CSR_HGEIP_ADDRESS => {
                self.hgeip = value;
            }
            CSR_HENVCFG_ADDRESS => {
                self.henvcfg = value;
            }
            CSR_HGATP_ADDRESS => {
                self.hgatp = value;
            }
            _ => {
                println!("This csr number isn't supported: {:#x}", csr_address);
                panic!();
            }
        }
    }
}

pub fn emulate_csr(
    hypervisor: &mut HypervisorContext,
    csr_address: usize,
    rd: usize,
    write_value: u64,
    registers: &mut [u64],
) {
    let virtual_csr = &mut hypervisor.csr;
    registers[rd] = virtual_csr.get_csr(csr_address);
    virtual_csr.set_csr(csr_address, write_value);

    if csr_address == CSR_HGATP_ADDRESS {
        let _ = shadow_map_address_stage2(true, true, true);
    }
}
