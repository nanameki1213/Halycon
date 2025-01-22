pub const COMPRESSION_FIELD: u32 = (1 << 2) - 1;
pub const COMPRESSION: u32 = 0x0;

pub const RD_OFFSET: u32 = 7;
pub const RD_MASK: u32 = ((1 << 5) - 1) << RD_OFFSET;

pub const RS2_OFFSET: u32 = 20;
pub const RS2_MASK: u32 = ((1 << 5) - 1) << RS2_OFFSET;
