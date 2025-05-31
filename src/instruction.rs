#![allow(dead_code)]

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
    const FUNCT12_OFFSET: u32 = 20;
    const FUNCT12_MASK: u32 = ((1 << 12) - 1) << Self::FUNCT12_OFFSET;
    const COMPRESSION_OFFSET: u32 = 0;
    const COMPRESSION_MASK: u32 = ((1 << 2) - 1) << Self::COMPRESSION_OFFSET;

    const OPCODE_CSR: usize = 0b1110011;
    const FUNCT3_CSRRS: usize = 0b010;
    const FUNCT3_CSRRW: usize = 0b001;
    const COMPRESSED_INSTRUCTION: usize = 0b01;

    pub const fn new(instruction: u32) -> Self {
        Self(instruction)
    }

    pub fn get_opcode(&mut self) -> usize {
        ((self.0 & Self::OPCODE_MASK) >> Self::OPCODE_OFFSET) as usize
    }

    pub fn get_rd(&mut self) -> usize {
        ((self.0 & Self::RD_MASK) >> Self::RD_OFFSET) as usize
    }

    pub fn get_funct3(&mut self) -> usize {
        ((self.0 & Self::FUNCT3_MASK) >> Self::FUNCT3_OFFSET) as usize
    }

    pub fn get_rs1(&mut self) -> usize {
        ((self.0 & Self::RS1_MASK) >> Self::RS1_OFFSET) as usize
    }

    pub fn get_rs2(&mut self) -> usize {
        ((self.0 & Self::RS2_MASK) >> Self::RS2_OFFSET) as usize
    }

    pub fn get_funct12(&mut self) -> usize {
        ((self.0 & Self::FUNCT12_MASK) >> Self::FUNCT12_OFFSET) as usize
    }

    pub fn get_compression(&mut self) -> usize {
        ((self.0 & Self::COMPRESSION_MASK) >> Self::COMPRESSION_OFFSET) as usize
    }

    pub fn is_csrrs_instruction(&mut self) -> bool {
        self.get_opcode() == Self::OPCODE_CSR && self.get_funct3() == Self::FUNCT3_CSRRS
    }

    pub fn is_compression_instruction(&mut self) -> bool {
        let compression = self.get_compression();
        compression == Self::COMPRESSED_INSTRUCTION
    }
}
