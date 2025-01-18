pub const SBI_EXT_BASE: u64 = 0x10;

pub const SBI_FID_PROBE_SBI_EXT: u64 = 3;

pub struct sbiret {
    pub error: u64,
    pub value: u64,
}
