use crate::println;
use arrayvec::ArrayVec;
use byteorder::{BigEndian, ByteOrder};
use core::{ffi::CStr, fmt, str::Utf8Error, usize};
use crate::MemoryEntry;

pub const FDT_MAGIC: u32 = 0xd00dfeed;
pub const FDT_VERSION: u32 = 17;

pub const FDT_BEGIN_NODE: u32 = 0x1;
pub const FDT_END_NODE: u32 = 0x2;
pub const FDT_PROP: u32 = 0x3;
pub const FDT_NOP: u32 = 0x4;
pub const FDT_END: u32 = 0x9;

pub enum FdtError {
    InvalidMagic,
    UnsupportedVersion,
    UnexpectedEOF,
    CapacityExceeded,
    StringError(core::str::Utf8Error),
}

impl fmt::Display for FdtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FdtError::InvalidMagic => write!(f, "FDT header contains invalid magic number."),
            FdtError::UnsupportedVersion => write!(f, "FDT version is not supported."),
            FdtError::UnexpectedEOF => write!(f, "FDT parse: Unexpected EOF."),
            FdtError::CapacityExceeded => write!(f, "FDT parse: capacity exceeded."),
            FdtError::StringError(err) => write!(f, "FDT parse: string error: {}.", err),
        }
    }
}

impl From<Utf8Error> for FdtError {
    fn from(value: Utf8Error) -> Self {
        FdtError::StringError(value)
    }
}

impl<T> From<arrayvec::CapacityError<T>> for FdtError {
    fn from(_: arrayvec::CapacityError<T>) -> Self {
        FdtError::CapacityExceeded
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
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
    struct_current_offset: usize,
    header: FdtHeader,
}

impl FdtContext {
    pub fn get_struct_block_current_address(&self) -> *const u32 {
        return (self.header.off_dt_struct as usize
            + self.fdt_address as usize
            + self.struct_current_offset * core::mem::size_of::<u32>())
            as *const u32;
    }

    pub fn get_struct_block_address(&self, offset: usize) -> *const u32 {
        return (self.header.off_dt_struct as usize
            + self.fdt_address as usize
            + offset * core::mem::size_of::<u32>()) as *const u32;
    }

    pub fn get_strings_block_address(&self) -> *const u32 {
        return (self.header.off_dt_strings as usize + self.fdt_address as usize) as *const u32;
    }

    pub fn contains_struct_block(&self) -> bool {
        self.struct_current_offset * core::mem::size_of::<u32>()
            <= self.header.size_dt_struct as usize
    }
}

#[derive(Debug)]
#[repr(C)]
struct FdtPropData {
    len: u32,
    nameoff: u32,
}

#[derive(Debug)]
struct FdtProp {
    name: &'static str,
    value: &'static [u8],
}

struct FdtNodeProps<const MAX_NODE_PROPS: usize> {
    props: ArrayVec<FdtProp, MAX_NODE_PROPS>,
}

pub fn get_cstr(ptr: *const u8) -> Result<&'static str, Utf8Error> {
    let c_str = unsafe { CStr::from_ptr(ptr as *const u8) };
    match c_str.to_str() {
        Ok(str) => {
            return Ok(str);
        }
        Err(err) => return Err(err),
    }
}

// TODO: MMIO関係の定義はmmioディレクトリ配下に移動する
pub struct MmioEntry {
    address: usize,
    size: usize,
}

pub struct DeviceTreeInfo<const MAX_MEMORY_ENTRIES: usize, const MAX_MMIO_ENTRIES: usize> {
    pub memory: ArrayVec<MemoryEntry, MAX_MEMORY_ENTRIES>,
    pub mmio: ArrayVec<MmioEntry, MAX_MMIO_ENTRIES>,
}

impl<const MAX_MEMORY_ENTRIES: usize, const MAX_MMIO_ENTRIES: usize>
    DeviceTreeInfo<MAX_MEMORY_ENTRIES, MAX_MMIO_ENTRIES>
{
    pub const fn new() -> Self {
        Self {
            memory: ArrayVec::new_const(),
            mmio: ArrayVec::new_const(),
        }
    }

    fn parse_header(&self, fdt_address: *const u32) -> FdtHeader {
        let ptr = fdt_address as *const u8;
        let buf = unsafe { core::slice::from_raw_parts(ptr, core::mem::size_of::<FdtHeader>()) };

        let header = FdtHeader {
            magic: BigEndian::read_u32(&buf[0..4]),
            totalsize: BigEndian::read_u32(&buf[4..8]),
            off_dt_struct: BigEndian::read_u32(&buf[8..12]),
            off_dt_strings: BigEndian::read_u32(&buf[12..16]),
            off_mem_rsvmap: BigEndian::read_u32(&buf[16..20]),
            version: BigEndian::read_u32(&buf[20..24]),
            last_comp_version: BigEndian::read_u32(&buf[24..28]),
            boot_cpuid_phys: BigEndian::read_u32(&buf[28..32]),
            size_dt_strings: BigEndian::read_u32(&buf[32..36]),
            size_dt_struct: BigEndian::read_u32(&buf[36..40]),
        };

        header
    }

    fn check_fdt_header(header: FdtHeader) -> Result<(), FdtError> {
        if header.magic != FDT_MAGIC {
            return Err(FdtError::InvalidMagic);
        }
        if header.version != FDT_VERSION {
            return Err(FdtError::UnsupportedVersion);
        }
        Ok(())
    }

    fn read_be32(address: *const u32) -> u32 {
        let byte = unsafe {
            core::slice::from_raw_parts(address as *const u8, core::mem::size_of::<u32>())
        };

        BigEndian::read_u32(byte)
    }

    fn get_prop_data(address: *const u32) -> FdtPropData {
        let buf = unsafe {
            core::slice::from_raw_parts(address as *const u8, core::mem::size_of::<FdtPropData>())
        };

        FdtPropData {
            len: BigEndian::read_u32(&buf[0..4]),
            nameoff: BigEndian::read_u32(&buf[4..8]),
        }
    }

    fn get_prop(
        fdt: &FdtContext,
        offset: &mut usize,
        prop_data: &FdtPropData,
    ) -> Result<FdtProp, FdtError> {
        let prop_name_ptr =
            (fdt.get_strings_block_address() as usize + prop_data.nameoff as usize) as *const u8;
        let prop_name = get_cstr(prop_name_ptr)?;
        let u32_size = core::mem::size_of::<u32>();

        *offset += core::mem::size_of::<FdtPropData>() / u32_size;
        let prop_value = unsafe {
            core::slice::from_raw_parts(
                fdt.get_struct_block_address(*offset) as *const u8,
                prop_data.len as usize,
            )
        };
        *offset += prop_data.len as usize / u32_size;

        Ok(FdtProp {
            name: prop_name,
            value: prop_value,
        })
    }

    fn search_prop<const MAX_NODE_PROPS: usize>(
        props: &FdtNodeProps<MAX_NODE_PROPS>,
        search_name: &'static str,
    ) -> Option<&'static [u8]> {
        for prop in props.props.iter() {
            if prop.name == search_name {
                return Some(prop.value);
            }
        }
        None
    }

    fn fdt_node_iteration(fdt: &mut FdtContext) -> Result<bool, FdtError> {
        assert!(fdt.contains_struct_block());
        loop {
            if !fdt.contains_struct_block() {
                return Err(FdtError::UnexpectedEOF);
            }
            let token = Self::read_be32(fdt.get_struct_block_current_address());
            fdt.struct_current_offset += 1;
            match token {
                FDT_BEGIN_NODE => return Ok(true),
                FDT_PROP => {
                    let prop_data = Self::get_prop_data(fdt.get_struct_block_current_address());
                    fdt.struct_current_offset += (core::mem::size_of::<FdtPropData>()
                        + prop_data.len as usize)
                        / core::mem::size_of::<u32>();
                }
                FDT_END => return Ok(false),
                _ => {}
            }
        }
    }

    fn fdt_prop_iteration(fdt: &FdtContext, offset: &mut usize) -> Result<bool, FdtError> {
        assert!(fdt.contains_struct_block());
        loop {
            if !fdt.contains_struct_block() {
                return Err(FdtError::UnexpectedEOF);
            }
            let token = Self::read_be32(fdt.get_struct_block_address(*offset));
            // println!(
            //     "offset: {:#X}, token: {}",
            //     fdt.header.off_dt_struct as usize + *offset * 4,
            //     token
            // );
            *offset += 1;
            match token {
                FDT_BEGIN_NODE => Self::skip_fdt_node(fdt, offset)?,
                FDT_PROP => return Ok(true),
                FDT_END_NODE => return Ok(false),
                _ => {}
            }
        }
    }

    fn scan_node_properties<const MAX_NODE_PROPS: usize>(
        fdt: &FdtContext,
        offset: &mut usize,
    ) -> Result<FdtNodeProps<MAX_NODE_PROPS>, FdtError> {
        let mut props: FdtNodeProps<MAX_NODE_PROPS> = FdtNodeProps {
            props: ArrayVec::new(),
        };
        while Self::fdt_prop_iteration(fdt, offset)? {
            // println!("({:#X})prop", fdt.header.off_dt_struct as usize + *offset * 4);
            let current_address = fdt.get_struct_block_address(*offset);
            let prop_data = Self::get_prop_data(current_address);
            let prop = Self::get_prop(fdt, offset, &prop_data)?;

            props.props.try_push(prop)?;
        }

        Ok(props)
    }

    fn skip_fdt_node(fdt: &FdtContext, offset: &mut usize) -> Result<(), FdtError> {
        let mut token = Self::read_be32(fdt.get_struct_block_current_address());
        while token != FDT_END_NODE {
            *offset += 1;
            token = Self::read_be32(fdt.get_struct_block_address(*offset));
            if token == FDT_BEGIN_NODE {
                Self::skip_fdt_node(fdt, offset)?;
            }
        }
        *offset += 1;

        Ok(())
    }

    pub fn parse(&mut self, fdt_address: *const u32) -> Result<(), FdtError> {
        let header = self.parse_header(fdt_address);
        let mut fdt_context = FdtContext {
            fdt_address,
            struct_current_offset: 0,
            header,
        };
        Self::check_fdt_header(header)?;

        const MAX_NODE_PROPS: usize = 32;

        while Self::fdt_node_iteration(&mut fdt_context)? {
            // println!(
            //     "BEGIN: {:#x}",
            //     fdt_context.header.off_dt_struct as usize + fdt_context.struct_current_offset * 4
            // );
            let mut prop_offset = fdt_context.struct_current_offset;
            let props =
                Self::scan_node_properties::<MAX_NODE_PROPS>(&fdt_context, &mut prop_offset)?;
            // println!("props: {:?}", props.props);
            if let Some(prop_value) = Self::search_prop(&props, "device_type") {
                if prop_value == "memory\0".as_bytes() {
                    if let Some(reg_value) = Self::search_prop(&props, "reg") {
                        let bytes: &[u8] = reg_value;
                        let address = BigEndian::read_u64(&bytes[0..8]);
                        let size = BigEndian::read_u64(&bytes[8..16]);
                        println!("memory: {:#X}, {:#X}", address, size);

                        self.memory.try_push(MemoryEntry {
                            address: address as usize,
                            size: size as usize,
                        })?
                    }
                }
            } else if let Some(prop_value) = Self::search_prop(&props, "compatible") {
                if prop_value == "virtio,mmio\0".as_bytes() {
                    if let Some(reg_value) = Self::search_prop(&props, "reg") {
                        let bytes: &[u8] = reg_value;
                        let address = BigEndian::read_u64(&bytes[0..8]);
                        let size = BigEndian::read_u64(&bytes[8..16]);
                        println!("virtio,mmio: {:#X}, {:#X}", address, size);

                        self.mmio.try_push(MmioEntry {
                            address: address as usize,
                            size: size as usize,
                        })?
                    }
                }
            }
        }

        Ok(())
    }
}
