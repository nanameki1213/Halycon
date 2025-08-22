use byteorder::{BigEndian, ByteOrder};
use crate::println;
use core::{ffi::CStr, mem::MaybeUninit, usize};
use crate::string_utils::hex_ptr_to_usize_length;

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

pub struct FdtContext {
    fdt_address: *const u32,
    header: FdtHeader,
}

impl FdtContext {
    pub fn get_struct_block_address(&self) -> *const u32 {
        return (self.header.off_dt_struct as usize + self.fdt_address as usize) as *const u32;
    }

    pub fn get_strings_block_address(&self) -> *const u32 {
        return (self.header.off_dt_strings as usize + self.fdt_address as usize) as *const u32;
    }

    pub fn contains_struct_block(&self, address: *const u32) -> bool {
        let start_address = self.get_struct_block_address();
        let end_address = (start_address as usize + self.header.size_dt_struct as usize) as *const u32;

        (start_address..end_address).contains(&address)
    }
}

#[derive(Debug)]
#[repr(C)]
struct FdtPropData {
    len: u32,
    nameoff: u32,
}

pub static mut HOST_FDT_CONTEXT: MaybeUninit<FdtContext> = MaybeUninit::<FdtContext>::uninit();

pub unsafe fn get_cstr(ptr: *const u8) -> Result<&'static str, ()> {
    let c_str = CStr::from_ptr(ptr as *const u8);
    match c_str.to_str() {
        Ok(str) => { return Ok(str); },
        Err(_) => { return Err(()) },
    }
}

// ホストのFDTを解析してメモリやペリフェラル情報を各地に通達
// 将来的にメモリやペリフェラルを一元管理できる仕組みができた際にはそれに合わせて
// より汎用的なプログラムにする。
pub unsafe fn parse_host_fdt(fdt_pointer: *const u32) -> Result<(), FdtError> {
    HOST_FDT_CONTEXT.as_mut_ptr().write(FdtContext {
        fdt_address: fdt_pointer as *const u32,
        header: parse_fdt_header(fdt_pointer),
    });

    let context = HOST_FDT_CONTEXT.assume_init_ref();
    let header = &context.header;

    match check_fdt_header(&header) {
        Ok(()) => {},
        Err(error) => return Err(error),
    }

    let mut current_address = context.get_struct_block_address();
    loop {
        let token = fdt_token_iteration(&context, &mut current_address)?;
        current_address = current_address.add(1);
        match token {
            FDT_BEGIN_NODE => {
                let unit_name = get_cstr(current_address as *const u8).unwrap();
                println!("unit_name: {}", unit_name);

                // メモリ情報を探索
                if let Some(prop_value) = search_fdt_property(&context, current_address, "device_type")? {
                    if prop_value == "memory\0" {
                        if let Some(reg_value) = search_fdt_property(&context, current_address, "reg")? {
                            println!("reg value: {:?}", reg_value.as_bytes());
                            let mut bytes = reg_value.as_bytes().as_ptr();
                            let address = hex_ptr_to_usize_length(bytes, 8).unwrap();
                            bytes = bytes.add(8);
                            let size = hex_ptr_to_usize_length(bytes, 8).unwrap();

                            println!("memory: {:#X}, {:#X}", address, size);
                        }
                    }
                }
            }
            FDT_END_NODE => {},
            FDT_NOP => {},
            FDT_END => { break; }
            _ => {},
        }
    }

    Ok(())
}

pub unsafe fn parse_fdt_header(fdt_pointer: *const u32) -> FdtHeader {
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

unsafe fn fdt_token_iteration(fdt: &FdtContext, current_address: &mut *const u32) -> Result<u32, FdtError> {
    assert!(fdt.contains_struct_block(*current_address));

    loop {
        if !fdt.contains_struct_block(*current_address) {
            return Err(FdtError::UnexpectedEOF);
        }

        let byte = core::slice::from_raw_parts(*current_address as *const u8, core::mem::size_of::<u32>());

        match BigEndian::read_u32(byte) {
            FDT_BEGIN_NODE  => { return Ok(FDT_BEGIN_NODE) },
            FDT_END_NODE    => { return Ok(FDT_END_NODE) },
            FDT_PROP        => { return Ok(FDT_PROP) },
            FDT_NOP         => { return Ok(FDT_NOP) },
            FDT_END         => { return Ok(FDT_END) },
            _ => {}
        }

        *current_address = (*current_address).add(1);
    }
}

unsafe fn skip_fdt_node(fdt: &FdtContext, mut node_address: *const u32) -> Result<*const u32, FdtError> {
    let mut token = fdt_token_iteration(fdt, &mut node_address)?;
    while token != FDT_END_NODE {
        node_address = node_address.add(1);
        token = fdt_token_iteration(fdt, &mut node_address)?;
        if token == FDT_BEGIN_NODE {
            node_address = skip_fdt_node(fdt, node_address)?;
        }
    }

    node_address = node_address.add(1);

    Ok(node_address)
}

unsafe fn search_fdt_property(fdt: &FdtContext, mut node_address: *const u32, search_name: &'static str) -> Result<Option<&'static str>, FdtError> {
    let mut token = fdt_token_iteration(fdt, &mut node_address)?;
    while token != FDT_END_NODE {
        node_address = node_address.add(1);
        if token == FDT_BEGIN_NODE {
            node_address = skip_fdt_node(&fdt, node_address)?;
        }
        if token == FDT_PROP {

            let ptr = node_address as *const u8;
            let buf = core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtPropData>());

            let prop_data = FdtPropData {
                len:        BigEndian::read_u32(&buf[0..4]),
                nameoff:    BigEndian::read_u32(&buf[4..8]),
            };

            let prop_name_ptr = (fdt.get_strings_block_address() as usize + prop_data.nameoff as usize) as *const u8;
            let prop_name = get_cstr(prop_name_ptr).unwrap();

            if prop_name == search_name {
                node_address = node_address.add(core::mem::size_of::<FdtPropData>() / 4); // prop_data分だけポインタを進める
                let prop_buf = core::slice::from_raw_parts(node_address as *const u8, prop_data.len as usize);
                let prop_value = str::from_utf8_unchecked(prop_buf);
                
                return Ok(Some(prop_value));
            }
        }
        token = fdt_token_iteration(fdt, &mut node_address)?;
    }

    Ok(None)
}

