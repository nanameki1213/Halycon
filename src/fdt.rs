use byteorder::{BigEndian, ByteOrder};

pub const FDT_MAGIC: u32 = 0xd00dfeed;
pub const FDT_VERSION: u32 = 17;

#[derive(Debug)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

pub unsafe fn parse_fdt_header(fdt_pointer: usize) -> Result<FdtHeader, ()> {
    let ptr = fdt_pointer as *const u8;
    let buf = core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtHeader>());

    let header = FdtHeader {
        magic:              BigEndian::read_u32(&buf[0..4]),
        totalsize:          BigEndian::read_u32(&buf[4..8]),
        off_dt_struct:      BigEndian::read_u32(&buf[8..12]),
        off_dt_strings:     BigEndian::read_u32(&buf[12..16]),
        off_mem_rsvmap:     BigEndian::read_u32(&buf[16..20]),
        version:            BigEndian::read_u32(&buf[20..24]),
        last_comp_version:  BigEndian::read_u32(&buf[24..28]),
        boot_cpuid_phys:    BigEndian::read_u32(&buf[28..32]),
        size_dt_strings:    BigEndian::read_u32(&buf[32..36]),
        size_dt_struct:     BigEndian::read_u32(&buf[36..40]),
    };

    Ok(header)
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
