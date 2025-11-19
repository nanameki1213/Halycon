use crate::paging;

use crate::MEMORY_ALLOCATOR;
use crate::println;
use arch::riscv::cpu::*;
use core::alloc::Layout;

pub fn allocate_pages(num_of_pages: usize, align: usize) -> *mut u8 {
    // TODO: Layoutのエラーハンドリング設計検討
    let layout =
        unsafe { Layout::from_size_align_unchecked(num_of_pages * paging::PAGE_SIZE, align) };

    match MEMORY_ALLOCATOR.lock().allocate(layout) {
        Ok(ptr) => ptr.as_ptr(),
        Err(_) => core::ptr::null_mut(),
    }
}

#[allow(dead_code)]
pub fn callocate_pages(num_of_pages: usize, align: usize) -> *mut u8 {
    let layout =
        unsafe { Layout::from_size_align_unchecked(num_of_pages * paging::PAGE_SIZE, align) };

    match MEMORY_ALLOCATOR.lock().callocate(layout) {
        Ok(ptr) => ptr.as_ptr(),
        Err(_) => core::ptr::null_mut(),
    }
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
