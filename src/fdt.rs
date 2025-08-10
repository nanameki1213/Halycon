use byteorder::{BigEndian, ByteOrder};

use crate::println;

pub const FDT_MAGIC: u32 = 0xd00dfeed;
pub const FDT_VERSION: u32 = 17;

pub const FDT_BEGIN_NODE: u32 = 0x1;
pub const FDT_END_NODE: u32 = 0x2;
pub const FDT_PROP: u32 = 0x3;
pub const FDT_NOP: u32 = 0x4;
pub const FDT_END: u32 = 0x9;

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

pub unsafe fn parse_fdt(fdt_pointer: usize) -> Result<(), &'static str> {
    let header = match parse_fdt_header(fdt_pointer) {
        Ok(header) => header,
        Err(_) => return Err("Cannnot parse fdt header"),
    };

    match check_fdt_header(&header) {
        Ok(()) => {},
        Err(msg) => return Err(msg),
    }

    let fdt_struct_pointer = (fdt_pointer as usize + header.off_dt_struct as usize) as *const u32;
    let mut current = fdt_struct_pointer;
    loop {
        let token = match fdt_token_iteration(&header, fdt_struct_pointer, &mut current) {
            Ok(token) => { token },
            Err(()) => { 0 },
        };
        current = current.add(1);
        println!("token: {:#X}", token);
        if token == FDT_END {
            break;
        }
    }

    Ok(())
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

pub fn check_fdt_header(header: &FdtHeader) -> Result<(), &'static str> {
    if header.magic != FDT_MAGIC {
        return Err("fdt magic value is invalid.");
    }

    if header.version != FDT_VERSION {
        return Err("fdt version is not supported.");
    }

    Ok(())
}

pub unsafe fn fdt_token_iteration(header: &FdtHeader, start_address: *const u32, current: &mut *const u32) -> Result<u32, ()> {
    let end_address = (start_address as usize + header.size_dt_struct as usize) as *const u32;

    if !(start_address..end_address).contains(current) {
        return Err(());
    }

    loop {
        let byte = core::slice::from_raw_parts(*current as *const u8, core::mem::size_of::<u32>());

        match BigEndian::read_u32(byte) {
            FDT_BEGIN_NODE  => { return Ok(FDT_BEGIN_NODE) },
            FDT_END_NODE    => { return Ok(FDT_END_NODE) },
            FDT_PROP        => { return Ok(FDT_PROP) },
            FDT_NOP         => { return Ok(FDT_NOP) },
            FDT_END         => { return Ok(FDT_END) },
            _ => {}
        }

        *current = (*current).add(1);
    }
}
