use core::arch::asm;

pub const MIE_MEIE_OFFSET: usize = 11;

pub const MTVEC_DIRECT: usize = 0;
pub const MTVEC_VECTORED: usize = 1;

pub const MISA_EXTENSION_H_OFFSET: usize = 7;

#[inline(always)]
pub fn get_hgatp() -> u64 {
    let hgatp: u64;
    unsafe { asm!("csrr {}, hgatp", out(reg) hgatp ) };
    hgatp
}

#[inline(always)]
pub fn set_hgatp(hgatp: u64) {
    unsafe { asm!("csrw hgatp, {}", in(reg) hgatp) };
}

#[inline(always)]
pub fn get_vsatp() -> u64 {
    let vsatp: u64;
    unsafe { asm!("csrr {}, vsatp", out(reg) vsatp ) };
    vsatp
}

#[inline(always)]
pub fn set_vsatp(vsatp: u64) {
    unsafe { asm!("csrw vsatp, {}", in(reg) vsatp ) };
}

#[inline(always)]
pub fn get_misa() -> u64 {
    let misa: u64;
    unsafe { asm!("csrr {}, misa", out(reg) misa ) };
    misa
}

#[inline(always)]
pub fn get_mie() -> u64 {
    let mie: u64;
    unsafe { asm!("csrr {}, mie", out(reg) mie ) };
    mie
}

#[inline(always)]
pub fn set_mie(mie: u64) {
    unsafe { asm!("csrw mie, {}", in(reg) mie ) };
}

#[inline(always)]
pub fn get_mtvec() -> u64 {
    let mtvec: u64;
    unsafe { asm!("csrr {}, mtvec", out(reg) mtvec ) };
    mtvec
}

#[inline(always)]
pub fn set_mtvec(mtvec: u64) {
    unsafe { asm!("csrw mtvec, {}", in(reg) mtvec ) };
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}
