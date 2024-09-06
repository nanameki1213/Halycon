use crate::paging;
use crate::println;

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
