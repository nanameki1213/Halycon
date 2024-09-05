use core::intrinsics;
use core::intrinsics::powf16;
use core::usize;

use crate::cpu::*;
use crate::println;

pub const DEFAULT_TABLE_LEVEL: i8 = 4;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

pub const PAGE_NUM_BITS: usize = 44;
pub const PAGE_TABLE_ENTRY_PPN: usize = ((1 << PAGE_NUM_BITS) - 1) << 10;

pub struct TableEntry(u64);

impl TableEntry {
    const PPN_MASK: usize = ((1 << PAGE_NUM_BITS) - 1) << 10;

    pub const fn new() -> Self {
        Self(0)
    }

    pub fn init(&mut self) {
        *self = Self::new();
    }
    
    pub fn get_next_table_address(&mut self) -> usize {
        return (self.0 & Self::PPN_MASK as u64) as usize;
   }
}

fn _map_address_stage2(
    physical_address: &mut usize,
    virtual_address: &mut usize,
    remaining_size: &mut usize,
    table_address: usize,
    permission: u64,
    table_level: i8,
    num_of_entries: usize,
) -> Result<(), ()> {
      Ok(()) 
}

pub fn map_address_stage2(
    mut physical_address: usize,
    mut virtual_address: usize,
    mut map_size: usize,
    is_readable: bool,
    is_writable: bool,
) -> Result<(), ()> {
    if (map_size & !PAGE_MASK) != 0 {
        println!("Map size is not aligned.");
        return Err(());
    }
    let hgatp = get_hgatp();
    println!("hgatp: {:#X}", hgatp);
    let page_table_address = (hgatp & HGATP_PPN_MASK as u64) as usize;
    let mode = ((hgatp & HGATP_MODE_MASK as u64) >> 60) as usize;

    let mut table_level: i8 = 0;
    match mode {
        0 => {
            println!("Virtual Memory not implemented.");
            return Err(());
        }
        8 => table_level = 3,
        9 => table_level = 4,
        10 => table_level = 5,
        _ => unreachable!(),
    }

    let table_address = unsafe { alloc_memory_for_paging(table_level).unwrap() };
    println!("table_address: {:#X}", table_address);

    let top_level_stage_2_num_of_entries = unsafe { powf16(2.0, 11.0) as usize };

    _map_address_stage2(
        &mut physical_address,
        &mut virtual_address,
        &mut map_size,
        table_address,
        0,
        table_level,
        top_level_stage_2_num_of_entries,
    );

    return Ok(());
}

pub fn init_stage_2_paging(table_level: i8) {
    if table_level == 0 {
        println!("Bare mode.");
        return;
    }
    let mut hgatp = get_hgatp();
    
    hgatp |= match table_level {
        3 => (0b1000 << 60),
        4 => (0b1001 << 60),
        5 => (0b1010 << 60),
        _ => unreachable!()
    };
    
    set_hgatp(hgatp);

}

unsafe extern "C" fn alloc_memory_for_paging(table_level: i8) -> Result<usize, ()> {
    extern "C" {
        static mut _free_area: u8;
    }
    Ok(&_free_area as *const u8 as usize)
}
