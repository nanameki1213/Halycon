use crate::{cpu::*, println};
use core::{arch::global_asm, usize};
use crate::instruction::*;
use crate::cpu::*;

pub const E_ILLEGAL_INSTRUCTION: usize = 0x2;

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
    lb a0, M_EXCEPTION
    mv a1, sp
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

.text
.extern S_EXCEPTION
.global supervisor_exception_handler 
.balign 256
supervisor_exception_handler:
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
    lb a0, S_EXCEPTION
    mv a1, sp
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

.text
.extern VS_EXCEPTION
.global virtual_supervisor_exception_handler 
.balign 256
virtual_supervisor_exception_handler:
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
    lb a0, VS_EXCEPTION
    mv a1, sp
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
    unsafe { set_mtvec((&machine_vector_table as *const _ as usize) as u64) }
    unsafe { set_stvec((&supervisor_vector_table as *const _ as usize) as u64) }
    unsafe { set_vstvec((&virtual_supervisor_vector_table as *const _ as usize) as u64) }
}

#[no_mangle]
pub fn exception_handler(mode: u8, sp: usize) {
    if mode == M_EXCEPTION {
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
    
    let scause = get_scause();

    if scause == E_ILLEGAL_INSTRUCTION as u64 {
        let stval = get_stval();
        instruction_abort(stval as u32, sp);
    }

    // エミュレーションし終わったので次の命令に進める
    let mut sepc = get_sepc();
    sepc += 4;
    set_sepc(sepc);
}

// 0xf1402573
fn instruction_abort(instruction: u32, sp: usize) {
    let opcode = instruction & OPCODE_MASK;

    match opcode {
        OPCODE_CSR => access_virtual_csr(instruction, sp),
        _ => unreachable!(),
    };
}

fn access_virtual_csr(instruction: u32, sp: usize) {
    let funct3 = (instruction & FUNCT3_MASK) >> FUNCT3_OFFSET;
    let csr_address = ((instruction & CSR_MASK) >> CSR_OFFSET) as usize;
    let rd_idx = ((instruction & RD_MASK) >> RD_OFFSET) as usize;
    let rs1_idx = ((instruction & RS1_MASK) >> RS1_OFFSET) as usize;

    let regs = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(sp as *mut u8, 16)
    };

    // read access
    if funct3 == FUNCT3_CSRRS {
        let ret = get_virtual_csr(csr_address);

        if rd_idx != REGISTER_ZERO { // format: csrrs rd,offset,zero
            match rd_idx {
                // a0 ~ a7
                REGISTER_A0..=REGISTER_A7 => regs[rd_idx - 9] = ret as u8, // TODO:
                                                                                    // ここらへんのマジック値をどうにかする
                // t0 ~ t6
                REGISTER_T0..=REGISTER_T2 => regs[rd_idx + 4] = ret as u8,
                REGISTER_T3..=REGISTER_T6 => regs[rd_idx - 16] = ret as u8,
                _ => unreachable!(),
            }
        }
        if rs1_idx != REGISTER_ZERO { // format: csrrs zero,offset,rs1
            let rs1 = match rs1_idx {
                // a0 ~ a7
                REGISTER_A0..=REGISTER_A7 => regs[rd_idx - 9], // TODO:
                                                                                    // ここらへんのマジック値をどうにかする
                // t0 ~ t6
                REGISTER_T0..=REGISTER_T2 => regs[rd_idx + 4],
                REGISTER_T3..=REGISTER_T6 => regs[rd_idx - 16],
                _ => unreachable!(),
            };
            set_virtual_csr(csr_address, ret | rs1 as u64);
        }
    } else if funct3 == FUNCT3_CSRRW {
        if rs1_idx != REGISTER_ZERO {
            let rs1 = match rs1_idx {
                // a0 ~ a7
                REGISTER_A0..=REGISTER_A7 => regs[rd_idx - 9], // TODO:
                                                                                    // ここらへんのマジック値をどうにかする
                // t0 ~ t6
                REGISTER_T0..=REGISTER_T2 => regs[rd_idx + 4],
                REGISTER_T3..=REGISTER_T6 => regs[rd_idx - 16],
                _ => unreachable!(),
            };
            set_virtual_csr(csr_address, rs1 as u64);
        }
        if rd_idx != REGISTER_ZERO {
            let ret = get_virtual_csr(csr_address);
            match rd_idx {
                // a0 ~ a7
                REGISTER_A0..=REGISTER_A7 => regs[rd_idx - 9] = ret as u8, // TODO:
                                                                                    // ここらへんのマジック値をどうにかする
                // t0 ~ t6
                REGISTER_T0..=REGISTER_T2 => regs[rd_idx + 4] = ret as u8,
                REGISTER_T3..=REGISTER_T6 => regs[rd_idx - 16] = ret as u8,
                _ => unreachable!(),
            }
        }
    }
}

fn get_virtual_csr(csr_address: usize) -> u64 {
    match csr_address {
        CSR_MHARTID_ADDRESS => 0,
        CSR_MIE_ADDRESS => 0,
        _ => unreachable!()
    }
}

fn set_virtual_csr(csr_address: usize, value: u64) {

}
