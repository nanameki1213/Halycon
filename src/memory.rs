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

pub fn set_pmp() {

    // for i in 0x3A0..0x3AF {
    //     let pmpcfg = get_csr(i);
    //     println!("pmpcfg{}: {:#X}", i, pmpcfg);
    // }

    // for i in 0x3B0..0x3EF {
    //     let pmpaddr = get_csr(i);
    //     println!("pmpaddr{}: {:#X}", i, pmpaddr);
    // }

    let mut pmpcfg0 = get_pmpcfg0();
    pmpcfg0 |= 0b00010111;
    set_pmpcfg0(pmpcfg0);

    let addr = 0x80000000;
    set_pmpaddr0(addr);
}
