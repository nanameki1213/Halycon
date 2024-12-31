use core::f16;
use core::intrinsics::powf16;
use core::intrinsics::unreachable;
use core::usize;

use crate::allocate_memory;
use crate::cpu::*;
use crate::println;

pub const DEFAULT_TABLE_LEVEL: i8 = 4;
pub const VPN_SIZE: i8 = 9;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

pub const PAGE_NUM_BITS: usize = 44;
pub const PAGE_TABLE_ENTRY_PPN: usize = ((1 << PAGE_NUM_BITS) - 1) << 10;

pub struct TableEntry(u64);

impl TableEntry {
    const PPN_OFFSET: usize = 10;
    const PPN_MASK: usize = ((1 << PAGE_NUM_BITS) - 1) << Self::PPN_OFFSET;
    const PERMISSION_OFFSET: usize = 10;
    const PERMISSION_MASK: usize = (1 << Self::PERMISSION_OFFSET) - 1;
    const V_OFFSET: usize = 0;
    const R_OFFSET: usize = 1;
    const W_OFFSET: usize = 2;
    const X_OFFSET: usize = 3;
    const U_OFFSET: usize = 4;
    const G_OFFSET: usize = 5;
    const A_OFFSET: usize = 6;
    const D_OFFSET: usize = 7;

    pub const fn new() -> Self {
        Self(0)
    }

    pub fn init(&mut self) {
        *self = Self::new();
    }

    pub fn get_next_table_address(&mut self) -> usize {
        return (self.0 & Self::PPN_MASK as u64) as usize;
    }

    pub fn set_output_address(&mut self, address: usize) {
        self.0 |= ((address & Self::PPN_MASK) << Self::PPN_OFFSET) as u64;
    }

    pub fn set_permission(&mut self, permission: u64) {
        self.0 |= permission & Self::PERMISSION_MASK as u64;
    }

    pub fn set_non_leaf_permission(&mut self) {
        self.0 |= (1 << Self::V_OFFSET) as u64;
        self.0 &= !((1 << Self::R_OFFSET) | (1 << Self::X_OFFSET)) as u64;
    }

    pub fn is_valid_pte(&mut self) -> bool {
        (self.0 & (1 << Self::V_OFFSET)) != 0
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
    let shift_level = 12 + 9 * table_level as usize;
    let table_index = (*virtual_address >> shift_level) & (num_of_entries - 1);
    let table = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(table_address as *mut TableEntry, num_of_entries)
    };

    // println!("func _map_address_stage2 {{");
    // println!("  table_level: {}", table_level);
    // println!("  table_address: {:#X}", table_address);
    // println!("  physical_address: {:#X}", *physical_address);
    // println!("  virtual_address : {:#X}", *virtual_address);
    // println!("  table_index: {}", table_index);
    // println!("}}\n");

    if table_level == 0 {
        for e in table[table_index..].iter_mut() {
            e.init();
            e.set_output_address(*physical_address);
            e.set_permission(permission);
            *physical_address += PAGE_SIZE;
            *virtual_address += PAGE_SIZE;
            *remaining_size -= PAGE_SIZE;
            if *remaining_size == 0 {
                return Ok(());
            }

            // println!("[debug]: leaf pte: {:#X}", e.0);
        }
        return Ok(());
    }

    for e in table[table_index..num_of_entries].iter_mut() {
        e.init();
        let mut next_table_address = e.get_next_table_address();
        if !e.is_valid_pte() {
            next_table_address = unsafe { allocate_memory(1, 0x1000).unwrap() };
            e.set_output_address(next_table_address);
            e.set_non_leaf_permission();

            // println!("[debug] pte: {:#X}", e.0);
        }

        let _ = _map_address_stage2(
            physical_address,
            virtual_address,
            remaining_size,
            next_table_address,
            permission,
            table_level - 1,
            num_of_entries,
        );

        if *remaining_size == 0 {
            return Ok(());
        }
    }
    return Ok(());
}

pub fn map_address_stage2(
    mut physical_address: usize,
    mut virtual_address: usize,
    mut map_size: usize,
    is_readable: bool,
    is_writable: bool,
    is_executable: bool,
) -> Result<(), ()> {
    if (map_size & PAGE_MASK) != 0 {
        println!("Map size is not aligned.");
        return Err(());
    }
    let vsatp = get_vsatp();
    // println!("vsatp: {:#X}", vsatp);
    let table_address = ((vsatp & VSATP_PPN_MASK as u64) << 12) as usize;
    let mode = ((vsatp & VSATP_MODE_MASK as u64) >> 60) as usize;

    let table_level: i8 = match mode {
        0 => {
            println!("Bare mode");
            return Err(());
        }
        8 => 3,
        9 => 4,
        10 => 5,
        _ => unreachable!(),
    };

    // println!("table_address: {:#X}", table_address);

    let top_level_stage_2_num_of_entries = 1 << VPN_SIZE;
        
    let mut permission: u64 = if is_readable {
        (1 << TableEntry::R_OFFSET) as u64
    } else {
        0
    };

    permission |= if is_writable {
        (1 << TableEntry::W_OFFSET) as u64
    } else {
        0
    };

    permission |= if is_executable {
        (1 << TableEntry::X_OFFSET) as u64
    } else {
        0
    };

    let _ = _map_address_stage2(
        &mut physical_address,
        &mut virtual_address,
        &mut map_size,
        table_address,
        permission,
        table_level - 1,
        top_level_stage_2_num_of_entries,
    );

    return Ok(());
}

pub fn init_stage_2_paging(table_level: i8) {
    if table_level == 0 {
        println!("Bare mode.");
        return;
    }
    let mut vsatp = get_vsatp();

    vsatp |= match table_level {
        3 => 0b1000 << 60,
        4 => 0b1001 << 60,
        5 => 0b1010 << 60,
        _ => unreachable!(),
    };

    let table_address = unsafe { allocate_memory(16, 1 << 14).unwrap() };
    // // ルート―ページテーブルは16KiBアラインメントしないといけないので14ビットずらす
    vsatp |= (table_address >> 12) as u64 & VSATP_PPN_MASK as u64;

    set_vsatp(vsatp);
}

unsafe extern "C" fn alloc_memory_for_paging() -> Result<usize, ()> {
    let address = allocate_memory(4, 1 << 12).unwrap();

    return Ok(address);
}
