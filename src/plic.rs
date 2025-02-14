use crate::cpu;

// TODO: get plic_addr from device tree
pub const PLIC_ADDR: usize = 0xc000000;

const PLIC_PRIORITY_OFFSET: usize = 0x0;
const PLIC_ENABLE_BITS_OFFSET: usize = 0x2000;

const PLIC_WORD_SIZE_BITS: usize = 32;
const PLIC_WORD_SIZE: usize = 4;
const PLIC_SOURCE_NUM: usize = 1024;
const PLIC_ENABLE_BITS_SIZE: usize = (PLIC_SOURCE_NUM / PLIC_WORD_SIZE_BITS) * PLIC_WORD_SIZE;

pub fn set_plic_enable(hart: usize, source: usize) {
    let row = source % 32;
    let column = source / 32;

    let plic_enable = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            (PLIC_ADDR + PLIC_ENABLE_BITS_OFFSET + hart * PLIC_ENABLE_BITS_SIZE) as *mut u32,
            PLIC_SOURCE_NUM / PLIC_WORD_SIZE_BITS
        )
    };

    plic_enable[row] = 1 << column;
}

pub fn set_plic_priority(source: usize, priority: usize) {
    let plic_priority = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            (PLIC_ADDR + PLIC_PRIORITY_OFFSET + source * PLIC_WORD_SIZE) as *mut u32,
            PLIC_SOURCE_NUM
        )
    };

    plic_priority[source] = priority as u32;
}

pub fn set_plic_thresholds(hart: usize, thresholds: usize) {
    let plic_thresholds_address = (PLIC_ADDR + 0x200000 + hart * 0x1000) as *mut u32;
    
    unsafe {
        core::ptr::write_volatile(plic_thresholds_address, thresholds as u32);
    }
}

// TODO: analyze from device tree
const UART_IRQ: usize = 0xa;
const PLIC_PRIORITY_MAX: usize = 7; // なんでわかる?

pub fn init_plic() {
    let hart = cpu::get_mhartid();

    set_plic_enable(hart, UART_IRQ);
    set_plic_priority(UART_IRQ, PLIC_PRIORITY_MAX);
    set_plic_thresholds(hart, 0);
}
