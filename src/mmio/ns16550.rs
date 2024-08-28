const NS16550_ADDR: usize = 0x10000000;
const NS16500_RBR: usize = 0x0;

pub fn putc(c: u8) {
    let reg = NS16550_ADDR;
    unsafe { core::ptr::write_volatile(reg as *mut u32, c as u32) }
}
