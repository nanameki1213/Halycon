use arch::riscv::cpu::{csr_address::*, get_time};
use spin::Mutex;
use crate::println;

pub struct VirtualCsr {
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

impl Default for VirtualCsr {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualCsr {
    pub const fn new() -> Self {
        VirtualCsr {
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

    pub fn get_csr(&mut self, csr_address: usize) -> u64 {
        match csr_address {
            CSR_TIME_ADDRESS => {
                get_time()
            }
            CSR_HSTATUS_ADDRESS => {
                self.hstatus
            },
            CSR_HEDELEG_ADDRESS => {
                self.hedeleg
            },
            CSR_HIDELEG_ADDRESS => {
                self.hideleg
            },
            CSR_HIE_ADDRESS => {
                self.hie
            },
            CSR_HCOUNTEREN_ADDRESS => {
                self.hcounteren
            },
            CSR_HGEIE_ADDRESS => {
                self.hgeie
            },
            CSR_HTVAL_ADDRESS => {
                self.htval
            },
            CSR_HIP_ADDRESS => {
                self.hip
            }, 
            CSR_HVIP_ADDRESS => {
                self.hvip
            },
            CSR_HTINST_ADDRESS => {
                self.htinst
            },
            CSR_HGEIP_ADDRESS => {
                self.hgeip
            },
            CSR_HENVCFG_ADDRESS => {
                self.henvcfg
            },
            CSR_HGATP_ADDRESS => {
                self.hgatp
            },
            _ => {
                println!("This csr number isn't supported: {:#x}", csr_address);
                panic!();
            }
        }
    }

    pub fn set_csr(&mut self, csr_address: usize, value: u64) {
        match csr_address {
            CSR_TIME_ADDRESS => {
                get_time(); // this register is read only.
            }
            CSR_HSTATUS_ADDRESS => {
                self.hstatus = value;
            },
            CSR_HEDELEG_ADDRESS => {
                self.hedeleg = value;
            },
            CSR_HIDELEG_ADDRESS => {
                self.hideleg = value;
            },
            CSR_HIE_ADDRESS => {
                self.hie = value;
            },
            CSR_HCOUNTEREN_ADDRESS => {
                self.hcounteren = value;
            },
            CSR_HGEIE_ADDRESS => {
                self.hgeie = value;
            },
            CSR_HTVAL_ADDRESS => {
                self.htval = value;
            },
            CSR_HIP_ADDRESS => {
                self.hip = value;
            }, 
            CSR_HVIP_ADDRESS => {
                self.hvip = value;
            },
            CSR_HTINST_ADDRESS => {
                self.htinst = value;
            },
            CSR_HGEIP_ADDRESS => {
                self.hgeip = value;
            },
            CSR_HENVCFG_ADDRESS => {
                self.henvcfg = value;
            },
            CSR_HGATP_ADDRESS => {
                self.hgatp = value;
            },
            _ => {
                println!("This csr number isn't supported: {:#x}", csr_address);
                panic!();
            }
        }
    }

}

pub static VIRTUAL_CSR: Mutex<VirtualCsr> = Mutex::new(VirtualCsr::new());

pub fn emulate_csr(csr_address: usize, rd: usize, write_value: u64, registers: &mut [u64]) {
    registers[rd] = VIRTUAL_CSR.lock().get_csr(csr_address);
    VIRTUAL_CSR.lock().set_csr(csr_address, write_value);
}
