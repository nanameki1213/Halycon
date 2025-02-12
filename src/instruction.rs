#![allow(dead_code)]

pub const OPCODE_MASK: u32 = (1 << 7) - 1;
pub const OPCODE_CSR: u32 = 0b1110011;

pub const CSR_OFFSET: u32 = 20;
pub const CSR_MASK: u32 = ((1 << 12) - 1) << CSR_OFFSET;

pub const FUNCT3_OFFSET: u32 = 12;
pub const FUNCT3_MASK: u32 = ((1 << 3) - 1) << FUNCT3_OFFSET;

pub const FUNCT3_CSRRS: u32 = 0b010;
pub const FUNCT3_CSRRW: u32 = 0b001;

pub const RD_OFFSET: u32 = 7;
pub const RD_MASK: u32 = ((1 << 5) - 1) << RD_OFFSET;

pub const RS2_OFFSET: u32 = 20;
pub const RS2_MASK: u32 = ((1 << 5) - 1) << RS2_OFFSET;

pub fn is_csrrs_instruction(instruction: u32) -> bool {
    (instruction & OPCODE_MASK == OPCODE_CSR) &&
    (((instruction & FUNCT3_MASK) >> FUNCT3_OFFSET) == FUNCT3_CSRRS)
}
