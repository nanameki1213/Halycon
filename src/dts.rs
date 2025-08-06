pub const FDT_MAGIC: u32 = 0xd00dfeed;
pub const FDT_VERSION: u32 = 17;

#[derive(Copy, Clone)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

pub fn check_fdt_header(header: FdtHeader) -> Result<(), &'static str> {
    if header.magic != FDT_MAGIC {
        return Err("fdt magic value is invalid.");
    }

    if header.version != FDT_VERSION {
        return Err("fdt version is not supported.");
    }

    Ok(())
}
