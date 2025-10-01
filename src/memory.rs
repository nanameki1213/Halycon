use core::usize;

use crate::cpu::*;
use crate::fdt::DeviceTreeInfo;
use crate::paging;
use crate::println;
use arrayvec::ArrayVec;

pub struct MemoryEntry {
    pub address: usize,
    pub size: usize,
}

pub struct MemoryAllocator {
    free_memory_entry: MemoryEntry,
    offset: usize,
}

impl MemoryAllocator {
    pub const fn new(memory_entry: &MemoryEntry) -> Self {
        extern "C" {
            static mut _free_area: u8;
        }
        let free_start_address = core::ptr::addr_of!(_free_area) as *const u8 as usize;

        let free_area = MemoryEntry {
            address: free_start_address,
            size: memory_entry.size - (free_start_address - memory_entry.address),
        };
        
        Self {
            free_memory_entry: free_area,
            offset: 0,
        }
    }

    pub fn get_free_memory_address(&self) -> usize {
        self.free_memory_entry.address + self.offset
    }

    pub fn allocate(&mut self, num_of_pages: usize, alignment: usize) -> Result<usize, ()> {
        let align_mask = alignment - 1;
        if (self.get_free_memory_address() & align_mask) != 0 {
            self.get_free_memory_address() &= !align_mask;
            self.get_free_memory_address() += alignment;
        }

        let top_address = self.free_start_address;
        self.free_start_address += paging::PAGE_SIZE * num_of_pages;
        if self.free_start_address - (self.memory_entries[0].address + self.free_area_offset) > self.memory_entries[0].size {
            Err(())
        } else {
            Ok(top_address)
        }
    }
}

pub static mut FREE_OFFSET: usize = 0;

pub unsafe extern "C" fn init_allocation<const MAX_MEMORY_ENTRIES: usize>(memory_entries: &ArrayVec<MemoryEntry, MAX_MEMORY_ENTRIES>) {
    extern "C" {
        static mut _free_area: u8;
    }
    let free_start_address = core::ptr::addr_of!(_free_area) as *const u8 as usize;

    if memory_entries.len() == 1 {
        let entry = &memory_entries[0];
        println!(
            "memory entry: address: {:#X}, size: {:#X}",
            entry.address, entry.size
        );
        if entry.address <= free_start_address && free_start_address < entry.address + entry.size {
            let free_area_offset = free_start_address - entry.address;
            let free_area_size = entry.size - free_area_offset;
            println!(
                "free area: address: {:#X}, size: {:#X}",
                free_start_address, free_area_size
            );
            return;
        } else {
            println!("no free area in the memory entry.");
            return;
        }
    } else {
        println!("multiple memory entries are not supported.");
        return;
    }
}

pub unsafe fn allocate_memory(num_of_pages: usize, alignment: usize) -> Result<usize, ()> {
    let align_mask = alignment - 1;
    // println!("FREE_ADDRESS: {:#X}", FREE_ADDRESS);
    // println!("align_mask: {:#X}", align_mask);
    if (FREE_ADDRESS & align_mask) != 0 {
        // println!("align: {:#X}", FREE_ADDRESS & align_mask);
        FREE_ADDRESS &= !align_mask;
        FREE_ADDRESS += alignment;
        // println!("after alignment address: {:#X}", FREE_ADDRESS);
    }

    let top_address = FREE_ADDRESS;
    FREE_ADDRESS += paging::PAGE_SIZE * num_of_pages;
    Ok(top_address)
}

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
