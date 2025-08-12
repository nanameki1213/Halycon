use byteorder::{BigEndian, ByteOrder};
use crate::println;
use core::{ffi::CStr, usize};

pub const FDT_MAGIC: u32 = 0xd00dfeed;
pub const FDT_VERSION: u32 = 17;

pub const FDT_BEGIN_NODE: u32 = 0x1;
pub const FDT_END_NODE: u32 = 0x2;
pub const FDT_PROP: u32 = 0x3;
pub const FDT_NOP: u32 = 0x4;
pub const FDT_END: u32 = 0x9;

pub enum FdtError {
    InvalidMagic = 100,
    UnsupportedVersion= 101,
    UnexpectedEOF = 102,
}

impl FdtError {
    pub fn as_str(&self) -> &'static str {
        match self {
            FdtError::InvalidMagic => "invalid header magic",
            FdtError::UnsupportedVersion => "unsupported version",
            FdtError::UnexpectedEOF => "unexpected EOF",
        }
    }
}

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

#[derive(Debug)]
#[repr(C)]
struct FdtPropData {
    len: u32,
    nameoff: u32,
}

pub unsafe fn get_cstr(ptr: *const u8) -> Result<&'static str, ()> {
    let c_str = CStr::from_ptr(ptr as *const u8);
    match c_str.to_str() {
        Ok(str) => { return Ok(str); },
        Err(_) => { return Err(()) },
    }
}

pub unsafe fn parse_fdt(fdt_pointer: usize) -> Result<(), FdtError> {
    let header = parse_fdt_header(fdt_pointer);

    match check_fdt_header(&header) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }

    let fdt_struct_pointer = (fdt_pointer as usize + header.off_dt_struct as usize) as *const u32;
    let mut current = fdt_struct_pointer;
    loop {
        let token = fdt_token_iteration(&header, fdt_struct_pointer, &mut current)?;
        current = current.add(1);
        match token {
            FDT_BEGIN_NODE => {
                let unit_name = get_cstr(current as *const u8).unwrap();
                println!("unit_name: {}", unit_name);
            },
            FDT_PROP => {
                let ptr = current as *const u8;
                let buf = core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtPropData>());
                
                let prop_data = FdtPropData {
                    len:        BigEndian::read_u32(&buf[0..4]),
                    nameoff:    BigEndian::read_u32(&buf[4..8]),
                };

                println!("{:?}", prop_data);
                let str_ptr = (fdt_pointer + header.off_dt_strings as usize + prop_data.nameoff as usize) as *const u8;
                let name = get_cstr(str_ptr).unwrap();
                
                current = current.add(core::mem::size_of::<FdtPropData>() / 4);
                let prop_buf = core::slice::from_raw_parts(current as *const u8, prop_data.len as usize);
                let prop_value = str::from_utf8_unchecked(prop_buf);

                println!("name: {}", name);
                println!("property value: {}", prop_value);
            },
            FDT_END_NODE => {},
            FDT_NOP => {},
            FDT_END => { break; }
            _ => {},
        }
    }

    Ok(())
}

pub unsafe fn parse_fdt_header(fdt_pointer: usize) -> FdtHeader {
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

    header
}

pub fn check_fdt_header(header: &FdtHeader) -> Result<(), FdtError> {
    if header.magic != FDT_MAGIC {
        return Err(FdtError::InvalidMagic);
    }

    if header.version != FDT_VERSION {
        return Err(FdtError::UnsupportedVersion);
    }

    Ok(())
}

unsafe fn fdt_token_iteration(header: &FdtHeader, start_address: *const u32, current: &mut *const u32) -> Result<u32, FdtError> {
    let end_address = (start_address as usize + header.size_dt_struct as usize) as *const u32;

    if !(start_address..end_address).contains(current) {
        panic!("precondition: fdt pointer is not pointing to fdt struct data");
    }

    loop {
        if *current == end_address {
            return Err(FdtError::UnexpectedEOF);
        }

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
