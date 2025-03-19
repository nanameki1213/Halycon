#![allow(dead_code)]

// TODO: get aplic_addr from device tree
pub const APLIC_ADDR: usize = 0xc000000;

const APLIC_DOMAINCFG_OFFSET: usize = 0x0;
const APLIC_SOURCECFG_OFFSET: usize = 0x4;
const APLIC_SETIE_OFFSET: usize     = 0x1e00;
const APLIC_IDC_OFFSET: usize       = 0x4000;

const APLIC_WORD_SIZE: usize = 4;

const APLIC_SOURCECFG_D_OFFSET: usize = 10;

const APLIC_DOMAINCFG_IE_OFFSET: usize = 8;
const APLIC_DOMAINCFG_DM_OFFSET: usize = 2;

const APLIC_IDC_IDELIVERY_OFFSET: usize = 0x0;

#[inline(always)]
fn set_aplic_by_offset(offset: usize, value: u32) {
    unsafe {
        core::ptr::write_volatile((APLIC_ADDR + offset) as *mut u32, value);
    }
}

pub fn set_plic_delegate(source: usize) {
    set_aplic_by_offset(APLIC_SOURCECFG_OFFSET + source * APLIC_WORD_SIZE, 1 << 10);
}

pub fn set_aplic_enable(source: usize) {
    let row = source % 32;
    let column = source / 32;

    set_aplic_by_offset(APLIC_SETIE_OFFSET + column, 1 << row);
}

// TODO: analyze from device tree
const UART_IRQ: usize = 0xa;

pub fn init_aplic() {
    set_aplic_by_offset(APLIC_DOMAINCFG_OFFSET, 1 << APLIC_DOMAINCFG_IE_OFFSET);

    // set_plic_delegate(UART_IRQ);
    set_aplic_enable(UART_IRQ);

    set_aplic_by_offset(APLIC_IDC_IDELIVERY_OFFSET + APLIC_IDC_IDELIVERY_OFFSET, 0x1);
}
