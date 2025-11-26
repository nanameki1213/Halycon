#![allow(dead_code)]

use core::arch::asm;

pub const MXLEN: usize = 64;

pub const MIE_MEIE_OFFSET: usize = 11;

pub const MSTATUS_SIE: usize = 1 << 1;
pub const MSTATUS_MIE: usize = 1 << 3;
pub const MSTATUS_MPP_0: usize = 1 << 11;
pub const MSTATUS_MPP_1: usize = 1 << 12;
pub const MSTATUS_MPRV: usize = 1 << 17;
pub const MSTATUS_TSR: usize = 1 << 22;
pub const MSTATUS_MPV: usize = 1 << 39;

pub const SSTATUS_SPP: usize = 1 << 8;

pub const HSTATUS_VSBE: usize = 1 << 5;
pub const HSTATUS_SPV: usize = 1 << 7;
pub const HSTATUS_SPVP: usize = 1 << 8;
pub const HSTATUS_VSTR: usize = 1 << 22;

pub const MIE_MEIE: usize = 1 << 11; // 外部割込み許可
pub const MIE_VSEIE: usize = 1 << 10;
pub const MIE_MTIE: usize = 1 << 7; // タイマ割込み許可
pub const MIE_MSIE: usize = 1 << 3; // ソフトウェア割込み許可

pub const XIE_SEIE: usize = 1 << 9; // 外部割込み許可(Sモード)
pub const XIE_STIE: usize = 1 << 5; // タイマ割込み許可(Sモード)
pub const XIE_SSIE: usize = 1 << 1; // ソフトウェア割込み許可(Sモード)

pub const MIP_MEIP: usize = 1 << 11; // 外部割込みペンディング
pub const MIP_VSEIP: usize = 1 << 10;
pub const MIP_MTIP: usize = 1 << 7; // タイマ割り込みペンディング
pub const MIP_MSIP: usize = 1 << 3; // ソフトウェア割込みペンディング

pub const XIP_SEIP: usize = 1 << 9; // 外部割込みペンディング(Sモード)
pub const XIP_STIP: usize = 1 << 5; // タイマ割込みペンディング(Sモード)
pub const XIP_SSIP: usize = 1 << 1; // ソフトウェア割込みペンディング(Sモード)

pub const TVEC_VECTORED: usize = 1;

pub const MISA_EXTENSION_H_OFFSET: usize = 7;
pub const MISA_MXL_OFFSET: usize = MXLEN - 2;
pub const MISA_MXL_MASK: usize = !((1 << MISA_MXL_OFFSET) - 1);

pub const SATP_PPN_MASK: usize = (1 << 44) - 1;
pub const SATP_MODE_MASK: usize = ((1 << 4) - 1) << 60;
pub const SATP_ASID_MASK: usize = ((1 << 14) - 1) << 44;

pub const PMP_1_CFG_OFFSET: usize = 8;
pub const PMP_A_FIELD_OFFSET: usize = 3;

pub const PMP_A_FIELD_TOR: usize = 1;
pub const PMP_A_FIELD_NA4: usize = 2;
pub const PMP_A_FIELD_NAPOT: usize = 3;

pub const ENVCFG_ADUE_OFFSET: usize = 61;

// CSRs address
pub mod csr_address {
    // Machine
    pub const CSR_MHARTID_ADDRESS: usize = 0xf14;
    pub const CSR_MIE_ADDRESS: usize = 0x304;
    pub const CSR_TIME_ADDRESS: usize = 0xc01;
    // Hypervisor Trap Setup
    pub const CSR_HSTATUS_ADDRESS: usize = 0x600;
    pub const CSR_HEDELEG_ADDRESS: usize = 0x602;
    pub const CSR_HIDELEG_ADDRESS: usize = 0x603;
    pub const CSR_HIE_ADDRESS: usize = 0x604;
    pub const CSR_HCOUNTEREN_ADDRESS: usize = 0x606;
    pub const CSR_HGEIE_ADDRESS: usize = 0x607;
    // Hypervisor Trap Handling
    pub const CSR_HTVAL_ADDRESS: usize = 0x643;
    pub const CSR_HIP_ADDRESS: usize = 0x644;
    pub const CSR_HVIP_ADDRESS: usize = 0x645;
    pub const CSR_HTINST_ADDRESS: usize = 0x64a;
    pub const CSR_HGEIP_ADDRESS: usize = 0xe12;
    // Hypervisor Configuration
    pub const CSR_HENVCFG_ADDRESS: usize = 0x60a;
    // Hypervisor Protection and Translation
    pub const CSR_HGATP_ADDRESS: usize = 0x680;

    pub fn is_hypervisor_csr(csr_number: usize) -> bool {
        (csr_number & 0xF00) == 0x600 || csr_number == CSR_HGEIP_ADDRESS
    }
}

// Registers
pub const REGISTER_ZERO: usize = 0;
pub const REGISTER_A0: usize = 10;
pub const REGISTER_A1: usize = 11;
pub const REGISTER_A2: usize = 12;
pub const REGISTER_A3: usize = 13;
pub const REGISTER_A4: usize = 14;
pub const REGISTER_A5: usize = 15;
pub const REGISTER_A6: usize = 16;
pub const REGISTER_A7: usize = 17;
pub const REGISTER_T0: usize = 5;
pub const REGISTER_T2: usize = 7;
pub const REGISTER_T3: usize = 28;
pub const REGISTER_T6: usize = 31;

// pub struct Registers {
//     pub x0: u64,
//     pub x1: u64,
//     pub x2: u64,
//     pub x3: u64,
//     pub x4: u64,
//     pub x5: u64,
//     pub x6: u64,
//     pub x7: u64,
//     pub x8: u64,
//     pub x9: u64,
//     pub x10: u64,
//     pub x11: u64,
//     pub x12: u64,
//     pub x13: u64,
//     pub x14: u64,
//     pub x15: u64,
//     pub x16: u64,
//     pub x17: u64,
//     pub x18: u64,
//     pub x19: u64,
//     pub x20: u64,
//     pub x21: u64,
//     pub x22: u64,
//     pub x23: u64,
//     pub x24: u64,
//     pub x25: u64,
//     pub x26: u64,
//     pub x27: u64,
//     pub x28: u64,
//     pub x29: u64,
//     pub x30: u64,
//     pub x31: u64,
// }

#[inline(always)]
pub fn get_xlen_from_misa() -> usize {
    let mxl = (get_misa() & MISA_MXL_MASK as u64) >> MISA_MXL_OFFSET as u64;
    match mxl {
        1 => 32,
        2 => 64,
        3 => 128,
        _ => 0,
    }
}

// mhartid

#[inline(always)]
pub fn get_mhartid() -> u64 {
    let mhartid: u64;
    unsafe { asm!("csrr {}, mhartid", out(reg) mhartid ) };
    mhartid
}

// mvendorid

#[inline(always)]
pub fn get_mvendorid() -> u64 {
    let mvendorid: u64;
    unsafe { asm!("csrr {}, mvendorid", out(reg) mvendorid ) };
    mvendorid
}

// misa

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

// status

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
pub fn get_hstatus() -> u64 {
    let hstatus: u64;
    unsafe { asm!("csrr {}, hstatus", out(reg) hstatus ) };
    hstatus
}

#[inline(always)]
pub fn set_hstatus(hstatus: u64) {
    unsafe { asm!("csrw hstatus, {}", in(reg) hstatus ) };
}

// ie

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
pub fn get_sie() -> u64 {
    let sie: u64;
    unsafe { asm!("csrr {}, sie", out(reg) sie ) };
    sie
}

#[inline(always)]
pub fn set_sie(sie: u64) {
    unsafe { asm!("csrw sie, {}", in(reg) sie ) };
}

#[inline(always)]
pub fn get_hie() -> u64 {
    let hie: u64;
    unsafe { asm!("csrr {}, hie", out(reg) hie ) };
    hie
}

#[inline(always)]
pub fn set_hie(hie: u64) {
    unsafe { asm!("csrw hie, {}", in(reg) hie ) };
}

// ip

#[inline(always)]
pub fn get_mip() -> u64 {
    let mip: u64;
    unsafe { asm!("csrr {}, mip", out(reg) mip ) };
    mip
}

#[inline(always)]
pub fn set_mip(mip: u64) {
    unsafe { asm!("csrw mip, {}", in(reg) mip ) };
}

#[inline(always)]
pub fn get_hip() -> u64 {
    let hip: u64;
    unsafe { asm!("csrr {}, hip", out(reg) hip ) };
    hip
}

#[inline(always)]
pub fn set_hip(hip: u64) {
    unsafe { asm!("csrw hip, {}", in(reg) hip ) };
}

// edeleg

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
pub fn get_hedeleg() -> u64 {
    let hedeleg: u64;
    unsafe { asm!("csrr {}, hedeleg", out(reg) hedeleg) };
    hedeleg
}

#[inline(always)]
pub fn set_hedeleg(hedeleg: u64) {
    unsafe { asm!("csrw hedeleg, {}", in(reg) hedeleg) };
}

// ideleg

#[inline(always)]
pub fn get_mideleg() -> u64 {
    let mideleg: u64;
    unsafe { asm!("csrr {}, mideleg", out(reg) mideleg) };
    mideleg
}

#[inline(always)]
pub fn set_mideleg(mideleg: u64) {
    unsafe { asm!("csrw mideleg, {}", in(reg) mideleg) };
}

#[inline(always)]
pub fn get_hideleg() -> u64 {
    let hideleg: u64;
    unsafe { asm!("csrr {}, hideleg", out(reg) hideleg) };
    hideleg
}

#[inline(always)]
pub fn set_hideleg(hideleg: u64) {
    unsafe { asm!("csrw hideleg, {}", in(reg) hideleg) };
}

// atp

#[inline(always)]
pub fn get_satp() -> u64 {
    let satp: u64;
    unsafe { asm!("csrr {}, satp", out(reg) satp ) };
    satp
}

#[inline(always)]
pub fn set_satp(satp: u64) {
    unsafe { asm!("csrw satp, {}", in(reg) satp ) };
}

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

// tvec

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
pub fn get_stvec() -> u64 {
    let stvec: u64;
    unsafe { asm!("csrr {}, stvec", out(reg) stvec ) };
    stvec
}

#[inline(always)]
pub fn set_stvec(stvec: u64) {
    unsafe { asm!("csrw stvec, {}", in(reg) stvec ) };
}

#[inline(always)]
pub fn get_vstvec() -> u64 {
    let vstvec: u64;
    unsafe { asm!("csrr {}, vstvec", out(reg) vstvec ) };
    vstvec
}

#[inline(always)]
pub fn set_vstvec(vstvec: u64) {
    unsafe { asm!("csrw vstvec, {}", in(reg) vstvec ) };
}

// epc

#[inline(always)]
pub fn get_mepc() -> u64 {
    let mepc: u64;
    unsafe { asm!("csrr {}, mepc", out(reg) mepc ) };
    mepc
}

#[inline(always)]
pub fn set_mepc(mepc: u64) {
    unsafe { asm!("csrw mepc, {}", in(reg) mepc ) };
}

#[inline(always)]
pub fn get_sepc() -> u64 {
    let sepc: u64;
    unsafe { asm!("csrr {}, sepc", out(reg) sepc ) };
    sepc
}

#[inline(always)]
pub fn set_sepc(sepc: u64) {
    unsafe { asm!("csrw sepc, {}", in(reg) sepc ) };
}

// tval

#[inline(always)]
pub fn get_mtval() -> u64 {
    let mtval: u64;
    unsafe { asm!("csrr {}, mtval", out(reg) mtval ) };
    mtval
}

#[inline(always)]
pub fn set_mtval(mtval: u64) {
    unsafe { asm!("csrw mtval, {}", in(reg) mtval) };
}

#[inline(always)]
pub fn get_stval() -> u64 {
    let stval: u64;
    unsafe { asm!("csrr {}, stval", out(reg) stval ) };
    stval
}

#[inline(always)]
pub fn set_stval(stval: u64) {
    unsafe { asm!("csrw stval, {}", in(reg) stval) };
}

#[inline(always)]
pub fn get_htval() -> u64 {
    let htval: u64;
    unsafe { asm!("csrr {}, htval", out(reg) htval ) };
    htval
}

#[inline(always)]
pub fn set_htval(htval: u64) {
    unsafe { asm!("csrw htval, {}", in(reg) htval) };
}

#[inline(always)]
pub fn get_vstval() -> u64 {
    let vstval: u64;
    unsafe { asm!("csrr {}, vstval", out(reg) vstval ) };
    vstval
}

// cause

#[inline(always)]
pub fn get_mcause() -> u64 {
    let mcause: u64;
    unsafe { asm!("csrr {}, mcause", out(reg) mcause ) };
    mcause
}

#[inline(always)]
pub fn set_mcause(mcause: u64) {
    unsafe { asm!("csrw mcause, {}", in(reg) mcause) };
}

#[inline(always)]
pub fn get_scause() -> u64 {
    let scause: u64;
    unsafe { asm!("csrr {}, scause", out(reg) scause ) };
    scause
}

#[inline(always)]
pub fn set_scause(scause: u64) {
    unsafe { asm!("csrw scause, {}", in(reg) scause) };
}

#[inline(always)]
pub fn get_vscause() -> u64 {
    let vscause: u64;
    unsafe { asm!("csrr {}, vscause", out(reg) vscause ) };
    vscause
}

#[inline(always)]
pub fn set_vscause(vscause: u64) {
    unsafe { asm!("csrw vscause, {}", in(reg) vscause) };
}

// scratch

#[inline(always)]
pub fn get_mscratch() -> u64 {
    let mscratch: u64;
    unsafe { asm!("csrr {}, mscratch", out(reg) mscratch ) };
    mscratch
}

#[inline(always)]
pub fn set_mscratch(mscratch: u64) {
    unsafe { asm!("csrw mscratch, {}", in(reg) mscratch) };
}

#[inline(always)]
pub fn get_sscratch() -> u64 {
    let sscratch: u64;
    unsafe { asm!("csrr {}, sscratch", out(reg) sscratch ) };
    sscratch
}

#[inline(always)]
pub fn set_sscratch(sscratch: u64) {
    unsafe { asm!("csrw sscratch, {}", in(reg) sscratch) };
}

// inst

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

// hvip

#[inline(always)]
pub fn get_hvip() -> u64 {
    let hvip: u64;
    unsafe { asm!("csrr {}, hvip", out(reg) hvip ) };
    hvip
}

#[inline(always)]
pub fn set_hvip(hvip: u64) {
    unsafe { asm!("csrw hvip, {}", in(reg) hvip ) };
}

// envcfg

#[inline(always)]
pub fn get_menvcfg() -> u64 {
    let menvcfg: u64;
    unsafe { asm!("csrr {}, menvcfg", out(reg) menvcfg ) };
    menvcfg
}

#[inline(always)]
pub fn set_menvcfg(menvcfg: u64) {
    unsafe { asm!("csrw menvcfg, {}", in(reg) menvcfg) };
}

#[inline(always)]
pub fn get_henvcfg() -> u64 {
    let henvcfg: u64;
    unsafe { asm!("csrr {}, henvcfg", out(reg) henvcfg ) };
    henvcfg
}

#[inline(always)]
pub fn set_henvcfg(henvcfg: u64) {
    unsafe { asm!("csrw henvcfg, {}", in(reg) henvcfg) };
}

// hcounteren

#[inline(always)]
pub fn get_hcounteren() -> u64 {
    let hcounteren: u64;
    unsafe { asm!("csrr {}, hcounteren", out(reg) hcounteren) };
    hcounteren
}

#[inline(always)]
pub fn set_hcounteren(hcounteren: u64) {
    unsafe { asm!("csrw hcounteren, {}", in(reg) hcounteren) };
}

// pmp

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

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}

// time

#[inline(always)]
pub fn get_time() -> u64 {
    let time: u64;
    unsafe { asm!("csrr {}, time", out(reg) time ) };
    time
}

// aia

#[inline(always)]
pub fn set_miselect(miselect: u64) {
    unsafe { asm!("csrw miselect, {}", in(reg) miselect ) };
}

#[inline(always)]
pub fn set_siselect(siselect: u64) {
    unsafe { asm!("csrw siselect, {}", in(reg) siselect ) };
}

#[inline(always)]
pub fn set_vsiselect(vsiselect: u64) {
    unsafe { asm!("csrw vsiselect, {}", in(reg) vsiselect ) };
}
