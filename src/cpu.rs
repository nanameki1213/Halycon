use core::arch::{asm, global_asm};

pub const MIE_MEIE_OFFSET: usize = 11;

pub const MTVEC_VECTORED: usize = 1;

pub const MISA_EXTENSION_H_OFFSET: usize = 7;

pub const HGATP_PPN_MASK: usize = (1 << 44) - 1;
pub const HGATP_MODE_MASK: usize = ((1 << 4) - 1) << 60;
pub const HGATP_VMID_MASK: usize = ((1 << 14) - 1) << 44;

pub const MSTATUS_TVM_OFFSET: usize = 20;

pub const PMP_1_CFG_OFFSET: usize = 8;
pub const PMP_A_FIELD_OFFSET: usize = 3;
pub const PMP_A_FIELD_TOR: usize = 1;

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
pub fn get_sstatus() -> u64 {
    let sstatus: u64;
    unsafe { asm!("csrr {}, sstatus", out(reg) sstatus ) };
    sstatus
}

#[inline(always)]
pub fn set_sstatus(sstatus: u64) {
    unsafe { asm!("csrw sstatus, {}", in(reg) sstatus ) };
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
pub fn get_medeleg() -> u64 {
    let medeleg: u64;
    unsafe { asm!("csrr {}, medeleg", out(reg) medeleg ) };
    medeleg
}

#[inline(always)]
pub fn set_medeleg(medeleg: u64) {
    unsafe { asm!("csrw medeleg, {}", in(reg) medeleg ) };
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
pub fn get_hedeleg() -> u64 {
    let hedeleg: u64;
    unsafe { asm!("csrr {}, hedeleg", out(reg) hedeleg) };
    hedeleg
}

#[inline(always)]
pub fn set_hedeleg(hedeleg: u64) {
    unsafe { asm!("csrw hedeleg, {}", in(reg) hedeleg) };
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
pub fn set_pmpaddr1(pmpaddr1: u64) {
    unsafe { asm!("csrw pmpaddr1, {}", in(reg) pmpaddr1) };
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
pub fn hfence() {
    unsafe { asm!("hfence.vvma
                   hfence.gvma") };
}

#[inline(always)]
pub fn hfence_vvma(vmid: usize) {
    unsafe { asm!("hfence.vvma {}", in(reg) vmid) };
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}
