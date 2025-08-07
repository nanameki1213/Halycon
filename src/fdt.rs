use byteorder::{BigEndian, LittleEndian, NativeEndian, ByteOrder};

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

pub unsafe fn parse_fdt_header(fdt_pointer: usize) -> Result<FdtHeader, ()> {
    let ptr = fdt_pointer as *const u8;
    let buf = core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtHeader>());

    let header = FdtHeader {
        magic:              LittleEndian::read_u32(&buf[0..2]),
        totalsize:          LittleEndian::read_u32(&buf[2..4]),
        off_dt_struct:      LittleEndian::read_u32(&buf[4..6]),
        off_mem_rsvmap:     LittleEndian::read_u32(&buf[6..8]),
        version:            LittleEndian::read_u32(&buf[8..10]),
        last_comp_version:  LittleEndian::read_u32(&buf[10..12]),
        boot_cpuid_phys:    LittleEndian::read_u32(&buf[12..14]),
        size_dt_strings:    LittleEndian::read_u32(&buf[14..16]),
        size_dt_struct:     LittleEndian::read_u32(&buf[16..18]),
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
