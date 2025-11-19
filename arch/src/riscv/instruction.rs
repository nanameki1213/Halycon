#![allow(dead_code)]

use crate::riscv::cpu::{HSTATUS_SPVP, get_hstatus, set_hstatus};
use core::arch::asm;

pub struct Instruction(u32);

impl Instruction {
    const OPCODE_MASK: u32 = (1 << 7) - 1;
    const OPCODE_OFFSET: u32 = 0;
    const RD_OFFSET: u32 = 7;
    const RD_MASK: u32 = ((1 << 5) - 1) << Self::RD_OFFSET;
    const FUNCT3_OFFSET: u32 = 12;
    const FUNCT3_MASK: u32 = ((1 << 3) - 1) << Self::FUNCT3_OFFSET;
    const RS1_OFFSET: u32 = 15;
    const RS1_MASK: u32 = ((1 << 5) - 1) << Self::RS1_OFFSET;
    const RS2_OFFSET: u32 = 20;
    const RS2_MASK: u32 = ((1 << 5) - 1) << Self::RS2_OFFSET;
    const FUNCT7_OFFSET: u32 = 25;
    const FUNCT7_MASK: u32 = ((1 << 7) - 1) << Self::FUNCT7_OFFSET;
    const FUNCT12_OFFSET: u32 = 20;
    const FUNCT12_MASK: u32 = ((1 << 12) - 1) << Self::FUNCT12_OFFSET;
    const COMPRESSION_OFFSET: u32 = 0;
    const COMPRESSION_MASK: u32 = ((1 << 2) - 1) << Self::COMPRESSION_OFFSET;

    const OPCODE_SYSTEM: usize = 0b1110011;
    const FUNCT3_CSRRS: usize = 0b010;
    const FUNCT3_CSRRW: usize = 0b001;
    const FUNCT7_SRET: usize = 0b0001000;
    const FUNCT7_MRET: usize = 0b0011000;
    const FUNCT7_MNRET: usize = 0b0111000;
    const RS2_TRAP_RETURN: usize = 0b00010;
    const COMPRESSED_INSTRUCTION: usize = 0b01;

    pub const fn new(instruction: u32) -> Self {
        Self(instruction)
    }

    pub fn get_opcode(&self) -> usize {
        ((self.0 & Self::OPCODE_MASK) >> Self::OPCODE_OFFSET) as usize
    }

    pub fn get_rd(&self) -> usize {
        ((self.0 & Self::RD_MASK) >> Self::RD_OFFSET) as usize
    }

    pub fn get_funct3(&self) -> usize {
        ((self.0 & Self::FUNCT3_MASK) >> Self::FUNCT3_OFFSET) as usize
    }

    pub fn get_rs1(&self) -> usize {
        ((self.0 & Self::RS1_MASK) >> Self::RS1_OFFSET) as usize
    }

    pub fn get_rs2(&self) -> usize {
        ((self.0 & Self::RS2_MASK) >> Self::RS2_OFFSET) as usize
    }
    
    pub fn get_funct7(&self) -> usize {
        ((self.0 & Self::FUNCT7_MASK) >> Self::FUNCT7_OFFSET) as usize
    }

    pub fn get_funct12(&self) -> usize {
        ((self.0 & Self::FUNCT12_MASK) >> Self::FUNCT12_OFFSET) as usize
    }

    pub fn get_compression(&mut self) -> usize {
        ((self.0 & Self::COMPRESSION_MASK) >> Self::COMPRESSION_OFFSET) as usize
    }

    pub fn is_valid_instruction(&mut self) -> bool {
        self.0 != 0
    }

    pub fn is_csrrw_instruction(&self) -> bool {
        self.get_opcode() == Self::OPCODE_SYSTEM && self.get_funct3() == Self::FUNCT3_CSRRW
    }

    pub fn is_csrrs_instruction(&self) -> bool {
        self.get_opcode() == Self::OPCODE_SYSTEM && self.get_funct3() == Self::FUNCT3_CSRRS
    }

    pub fn is_sret(&self) -> bool {
        self.get_opcode() == Self::OPCODE_SYSTEM && self.get_rs2() == Self::RS2_TRAP_RETURN && self.get_funct7() == Self::FUNCT7_SRET
    }

    pub fn is_compression_instruction(&mut self) -> bool {
        let compression = self.get_compression();
        compression == Self::COMPRESSED_INSTRUCTION
    }
}

// Hypervisor Instruction
fn hstatus_effective_privileged(is_vu: bool) {
    let mut hstatus = get_hstatus();
    if is_vu {
        hstatus &= !HSTATUS_SPVP as u64;
    } else {
        hstatus |= HSTATUS_SPVP as u64;
    }
    set_hstatus(hstatus);
}

pub fn read_vm_memory(is_vu: bool, address: usize, width: usize) -> u64 {
    hstatus_effective_privileged(is_vu);
    match width {
        64 => unsafe {
            let output: u64;
            asm!("
                .option arch, +h
                hlv.d {0}, 0({1})",
                out(reg) output,
                in(reg) address
            );
            output
        },
        32 => unsafe {
            let output: u32;
            asm!("
                .option arch, +h
                hlv.w {0}, 0({1})",
                out(reg) output,
                in(reg) address
            );
            output as u64
        },
        _ => 0,
    }
}

pub fn write_vm_memory(is_vu: bool, address: usize, value: u64, width: usize) {
    hstatus_effective_privileged(is_vu);
    match width {
        64 => unsafe {
            asm!("
                .option arch, +h
                hsv.d {0}, 0({1})",
                in(reg) value,
                in(reg) address
            );
        },
        32 => unsafe {
            asm!("
                .option arch, +h
                hsv.w {0}, 0({1})",
                in(reg) value as u32,
                in(reg) address
            );
        },
        _ => {}
    }
}
