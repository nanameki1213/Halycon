use crate::paging;
use crate::println;
use crate::cpu::*;

pub static mut FREE_ADDRESS: usize = 0;

pub unsafe extern "C" fn init_allocation() {
    extern "C" {
        static mut _free_area: u8;
    }
    FREE_ADDRESS = core::ptr::addr_of!(_free_area) as *const u8 as usize;
}

pub unsafe fn allocate_memory(num_of_pages: usize) -> Result<usize, ()> {
    if FREE_ADDRESS == 0 {
        println!("memory allocater is not initialized.");
        return Err(());
    }

    let top_address = FREE_ADDRESS;
    FREE_ADDRESS += paging::PAGE_SIZE * num_of_pages;
    Ok(top_address)
}

pub fn set_pmp(top_address: usize, bottom_address: usize,
               is_readable: bool, is_writable: bool, is_executable: bool) {
    let pmp1cfg = (is_readable as u8) << 0 |
                  (is_writable as u8) << 1 |
                  (is_executable as u8) << 2 |
                  (PMP_A_FIELD_TOR as u8) << PMP_A_FIELD_OFFSET;

    set_pmpcfg0(pmp1cfg as u64);
    set_pmpaddr0(bottom_address as u64);
    set_pmpaddr1(top_address as u64);
}
