use core::borrow::BorrowMut;
use core::usize;

use crate::cpu::*;
use crate::memory::allocate_pages;
use crate::println;

pub const DEFAULT_TABLE_LEVEL: i8 = 4;
pub const VPN_SIZE: i8 = 9;
pub const G_STAGE_TOP_VPN_SIZE: i8 = 11;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const PAGE_MASK: usize = PAGE_SIZE - 1;

pub const PAGE_NUM_BITS: usize = 44;

pub struct TableEntry(u64);

#[allow(dead_code)]
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
        return (((self.0 & Self::PPN_MASK as u64) >> Self::PPN_OFFSET as u64) << PAGE_SHIFT)
            as usize;
    }

    pub fn set_output_address(&mut self, address: usize) {
        self.0 |= (((address >> PAGE_SHIFT) << Self::PPN_OFFSET) & Self::PPN_MASK) as u64;
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

fn _resolve_address_stage2(
    virtual_address: usize,
    table_address: usize,
    table_level: i8,
    num_of_entries: usize,
) -> Result<usize, ()> {
    let shift_level = 12 + 9 * table_level as usize;
    let table_index = (virtual_address >> shift_level) & (num_of_entries - 1);
    let table = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(table_address as *mut TableEntry, num_of_entries)
    };

    let pte = table[table_index].borrow_mut();
    if !pte.is_valid_pte() {
        return Err(());
    }

    if table_level == 0 {
        let offset = virtual_address & ((1 << PAGE_SHIFT) - 1);
        return Ok(pte.get_next_table_address() + offset);
    }

    let next_table_address = pte.get_next_table_address();
    _resolve_address_stage2(
        virtual_address,
        next_table_address,
        table_level - 1,
        (1 << VPN_SIZE) as usize,
    )
}

#[allow(dead_code)]
pub fn resolve_address_stage2(virtual_address: usize) -> Result<usize, ()> {
    let hgatp = get_hgatp();
    let table_address = ((hgatp & SATP_PPN_MASK as u64) << 12) as usize;
    let mode = ((hgatp & SATP_MODE_MASK as u64) >> 60) as usize;

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

    let top_level_stage_2_num_of_entries = 1 << G_STAGE_TOP_VPN_SIZE;

    let physical_address = _resolve_address_stage2(
        virtual_address,
        table_address,
        table_level - 1,
        top_level_stage_2_num_of_entries,
    )?;
    Ok(physical_address)
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

    if table_level == 0 {
        for e in table[table_index..num_of_entries].iter_mut() {
            e.init();
            e.set_output_address(*physical_address);
            e.set_permission(
                permission | (1 << TableEntry::V_OFFSET) | (1 << TableEntry::U_OFFSET),
            );
            *physical_address += PAGE_SIZE;
            *virtual_address += PAGE_SIZE;
            *remaining_size -= PAGE_SIZE;

            if *remaining_size == 0 {
                return Ok(());
            }
        }
        return Ok(());
    }

    for e in table[table_index..num_of_entries].iter_mut() {
        e.init();
        let mut next_table_address = e.get_next_table_address();
        if !e.is_valid_pte() {
            // TODO: nullの可能性があるのでエラーハンドリング
            next_table_address = allocate_pages(1, PAGE_SIZE) as usize;
            e.set_output_address(next_table_address);
            e.set_non_leaf_permission();
        }

        let _ = _map_address_stage2(
            physical_address,
            virtual_address,
            remaining_size,
            next_table_address,
            permission,
            table_level - 1,
            (1 << VPN_SIZE) as usize,
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
    table_level: i8,
    is_readable: bool,
    is_writable: bool,
    is_executable: bool,
) -> Result<usize, ()> {
    if (map_size & PAGE_MASK) != 0 {
        println!("Map size is not aligned.");
        return Err(());
    }
    // TODO: nullの可能性があるのでエラーハンドリング
    let table_address = allocate_pages(4, 1 << 14) as usize;

    let top_level_stage_2_num_of_entries = 1 << G_STAGE_TOP_VPN_SIZE;

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

    return Ok(table_address as usize);
}
