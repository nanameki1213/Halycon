use core::arch::{asm, global_asm};

pub const MIE_MEIE_OFFSET: usize = 11;

pub const MTVEC_VECTORED: usize = 1;

pub const MISA_EXTENSION_H_OFFSET: usize = 7;

pub const HGATP_PPN_MASK: usize = (1 << 44) - 1;
pub const HGATP_MODE_MASK: usize = ((1 << 4) - 1) << 60;

pub const MSTATUS_TVM_OFFSET: usize = 20;

// #[inline(always)]
// pub fn get_csr(csr_addr: usize) -> u64 {
//     let csr: u64;
//     unsafe { asm!("csrr {tmp}, {number}", tmp = out(reg) csr, number = const csr_addr as *const _ as usize) }
//     csr
// }
// 
// #[inline(always)]
// pub fn set_csr(csr_addr: usize, csr_value: u64) {
//     unsafe { asm!("csrw {number}, {tmp}", number = in(reg) csr_addr, tmp = in(reg) csr_value) }
// }

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

#[inline(always)]
pub fn get_mstatus() -> u64 {
    let mstatus: u64;
    unsafe { asm!("csrr {}, mstatus", out(reg) mstatus ) };
    mstatus
}

#[inline(always)]
pub fn set_mstatus(mstatus: u64) {
    unsafe { asm!("csrw mstatus, {}", in(reg) mstatus ) };
}

#[inline(always)]
pub fn get_misa() -> u64 {
    let misa: u64;
    unsafe { asm!("csrr {}, misa", out(reg) misa ) };
    misa
}

#[inline(always)]
pub fn set_misa(misa: u64) {
    unsafe { asm!("csrw misa, {}", in(reg) misa ) };
}

#[inline(always)]
pub fn get_mtinst() -> u64 {
    let mtinst: u64;
    unsafe { asm!("csrr {}, mtinst", out(reg) mtinst ) };
    mtinst
}

#[inline(always)]
pub fn set_mtinst(mtinst: u64) {
    unsafe { asm!("csrw mtinst, {}", in(reg) mtinst ) };
}

#[inline(always)]
pub fn get_htinst() -> u64 {
    let htinst: u64;
    unsafe { asm!("csrr {}, htinst", out(reg) htinst ) };
    htinst
}

#[inline(always)]
pub fn set_htinst(htinst: u64) {
    unsafe { asm!("csrw htinst, {}", in(reg) htinst ) };
}

#[inline(always)]
pub fn get_pmpcfg0() -> u64 {
    let pmpcfg0: u64;
    unsafe { asm!("csrr {}, pmpcfg0", out(reg) pmpcfg0 ) };
    pmpcfg0
}

#[inline(always)]
pub fn set_pmpcfg0(pmpcfg0: u64) {
    unsafe { asm!("csrw pmpcfg0, {}", in(reg) pmpcfg0 ) };
}

#[inline(always)]
pub fn get_pmpaddr0() -> u64 {
    let pmpaddr0: u64;
    unsafe { asm!("csrr {}, pmpaddr0", out(reg) pmpaddr0 ) };
    pmpaddr0
}

#[inline(always)]
pub fn set_pmpaddr0(pmpaddr0: u64) {
    unsafe { asm!("csrw pmpaddr0, {}", in(reg) pmpaddr0 ) };
}

#[inline(always)]
pub fn get_pmpcfg2() -> u64 {
    let pmpcfg2: u64;
    unsafe { asm!("csrr {}, pmpcfg2", out(reg) pmpcfg2 ) };
    pmpcfg2
}

#[inline(always)]
pub fn set_pmpcfg2(pmpcfg2: u64) {
    unsafe { asm!("csrw pmpcfg2, {}", in(reg) pmpcfg2 ) };
}

#[inline(always)]
pub fn get_pmpcfg4() -> u64 {
    let pmpcfg4: u64;
    unsafe { asm!("csrr {}, pmpcfg4", out(reg) pmpcfg4 ) };
    pmpcfg4
}

#[inline(always)]
pub fn set_pmpcfg4(pmpcfg4: u64) {
    unsafe { asm!("csrw pmpcfg4, {}", in(reg) pmpcfg4 ) };
}

#[inline(always)]
pub fn get_pmpcfg6() -> u64 {
    let pmpcfg6: u64;
    unsafe { asm!("csrr {}, pmpcfg6", out(reg) pmpcfg6 ) };
    pmpcfg6
}

#[inline(always)]
pub fn set_pmpcfg6(pmpcfg6: u64) {
    unsafe { asm!("csrw pmpcfg6, {}", in(reg) pmpcfg6 ) };
}

#[inline(always)]
pub fn get_pmpcfg8() -> u64 {
    let pmpcfg8: u64;
    unsafe { asm!("csrr {}, pmpcfg8", out(reg) pmpcfg8 ) };
    pmpcfg8
}

#[inline(always)]
pub fn set_pmpcfg8(pmpcfg8: u64) {
    unsafe { asm!("csrw pmpcfg8, {}", in(reg) pmpcfg8 ) };
}

#[inline(always)]
pub fn get_pmpcfg10() -> u64 {
    let pmpcfg10: u64;
    unsafe { asm!("csrr {}, pmpcfg10", out(reg) pmpcfg10 ) };
    pmpcfg10
}

#[inline(always)]
pub fn set_pmpcfg10(pmpcfg10: u64) {
    unsafe { asm!("csrw pmpcfg10, {}", in(reg) pmpcfg10 ) };
}

#[inline(always)]
pub fn get_pmpcfg12() -> u64 {
    let pmpcfg12: u64;
    unsafe { asm!("csrr {}, pmpcfg12", out(reg) pmpcfg12 ) };
    pmpcfg12
}

#[inline(always)]
pub fn set_pmpcfg12(pmpcfg12: u64) {
    unsafe { asm!("csrw pmpcfg12, {}", in(reg) pmpcfg12 ) };
}

#[inline(always)]
pub fn get_pmpcfg14() -> u64 {
    let pmpcfg14: u64;
    unsafe { asm!("csrr {}, pmpcfg14", out(reg) pmpcfg14 ) };
    pmpcfg14
}

#[inline(always)]
pub fn set_pmpcfg14(pmpcfg14: u64) {
    unsafe { asm!("csrw pmpcfg14, {}", in(reg) pmpcfg14 ) };
}

#[inline(always)]
pub fn sfence() {
    unsafe { asm!("hfence.vvma
                   hfence.gvma") };
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}
