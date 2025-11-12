use arch::riscv::{cpu::{csr_address::*, get_time}, instruction::Instruction};

pub fn emulate_csr(csr_number: usize, instruction: Instruction registers: &mut [u64]) {
    match csr_number {
        CSR_TIME_ADDRESS => {
            let register_number = instruction.get_rd();
            registers[register_number] = get_time();
        }
        CSR_HSTATUS_ADDRESS => {},
        CSR_HEDELEG_ADDRESS => {},
        CSR_HIE_ADDRESS => {},
        CSR_HCOUNTEREN_ADDRESS => {},
        CSR_HGEIE_ADDRESS => {},
        CSR_HTVAL_ADDRESS => {},
        CSR_HIP_ADDRESS => {}, 
        CSR_HVIP_ADDRESS => {},
        CSR_HTINST_ADDRESS => {},
        CSR_HGEIP_ADDRESS => {},
        CSR_HENVCFG_ADDRESS => {},
        CSR_HGATP_ADDRESS => {},
        _ => {
            println!("This csr number isn't supported: {:#x}", csr_number);
            panic!();
        }
    }
}
