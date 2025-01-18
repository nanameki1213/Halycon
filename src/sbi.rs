pub const SBI_EXT_BASE: u64 = 0x10;

pub const SBI_FID_GET_SBI_IMPLEMENTATION_VERSION: u64 = 2;
pub const SBI_FID_PROBE_SBI_EXT: u64 = 3;

pub struct Sbiret {
    pub error: u64,
    pub value: u64,
}
