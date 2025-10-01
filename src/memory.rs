use core::usize;

use crate::cpu::*;
use crate::paging;
use crate::println;

pub struct MemoryEntry {
    pub address: usize,
    pub size: usize,
}

pub struct MemoryAllocator {
    free_memory_entry: MemoryEntry,
    free_address: usize,
}

pub enum MemoryAllocatorError {
    OutOfMemory,
}

impl core::fmt::Debug for MemoryAllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryAllocatorError::OutOfMemory => write!(f, "OutOfMemory"),
        }
    }
}

impl MemoryAllocator {
    pub const fn new() -> Self {
        MemoryAllocator {
            free_memory_entry: MemoryEntry {
                address: 0,
                size: 0,
            },
            free_address: 0,
        }
    }

    pub fn init(&mut self, memory_entry: &MemoryEntry) {
        self.free_memory_entry = MemoryEntry {
            address: memory_entry.address,
            size: memory_entry.size,
        };
        extern "C" {
            static mut _free_area: u8;
        }
        let free_start_address = core::ptr::addr_of!(_free_area) as *const u8 as usize;
        self.free_address = free_start_address;
    } 

    pub fn allocate(&mut self, num_of_pages: usize, alignment: usize) -> Result<usize, MemoryAllocatorError> {
        let align_mask = alignment - 1;
        if (self.free_address & align_mask) != 0 {
            self.free_address &= !align_mask;
            self.free_address += alignment;
        }

        let top_address = self.free_address;
        self.free_address += paging::PAGE_SIZE * num_of_pages;
        if self.free_address > self.free_memory_entry.address + self.free_memory_entry.size {
            Err(MemoryAllocatorError::OutOfMemory)
        } else {
            Ok(top_address)
        }
    }
}

pub static mut MEMORY_ALLOCATOR: MemoryAllocator = MemoryAllocator::new();

#[allow(dead_code)]
pub fn set_pmp(
    top_address: usize,
    bottom_address: usize,
    is_readable: bool,
    is_writable: bool,
    is_executable: bool,
) {
    let pmp1cfg = (is_readable as u8) << 0
        | (is_writable as u8) << 1
        | (is_executable as u8) << 2
        | (PMP_A_FIELD_TOR as u8) << PMP_A_FIELD_OFFSET;

    set_pmpcfg0(((pmp1cfg as u64) << 8) as u64);
    println!("[setup] pmpcfg0: {:#X}", get_pmpcfg0());
    set_pmpaddr0((bottom_address >> 2) as u64);
    set_pmpaddr1((top_address >> 2) as u64);
}

#[allow(dead_code)]
pub fn set_pmp_all_physical_address(is_readable: bool, is_writable: bool, is_executable: bool) {
    let pmp0cfg = (is_readable as u8) << 0
        | (is_writable as u8) << 1
        | (is_executable as u8) << 2
        | (PMP_A_FIELD_NAPOT as u8) << PMP_A_FIELD_OFFSET;
    if get_xlen_from_misa() == 64 {
        let pmpaddr0 = (1 << (56 - 2)) - 1;
        set_pmpaddr0(pmpaddr0);
        set_pmpcfg0(pmp0cfg as u64);
    }
}
