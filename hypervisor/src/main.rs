#![feature(riscv_ext_intrinsics)]
#![no_std]
#![no_main]

extern crate alloc;

mod aplic;
mod console;
mod loader;
mod memory;
mod paging;
mod plic;
mod sbi;
mod vector;
mod emulate_csr;
mod virtio_blk;
mod vm;
mod mmio {
    pub mod ns16550;
    pub mod virtio;
}

use alloc::vec::Vec;
use arch::riscv::cpu::*;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use fdt::DeviceTreeInfo;
use log;
use memory::set_pmp_all_physical_address;
use mmio::ns16550::Uart;
use mmio::virtio::VirtioMmio;
use spin::Mutex;
use string_utils::hex_ptr_to_usize;
use vector::setup_vector;

pub struct UartLogger;

impl log::Log for UartLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: UartLogger = UartLogger;

#[macro_export]
macro_rules! bitmask {
    ($high:expr,$low:expr) => {
        ((1 << (($high - $low) + 1)) - 1) << $low
    };
}

const MAX_MEMORY_ENTRIES: usize = 32;
const MAX_MMIO_ENTRIES: usize = 64;

// fn intr_disable() {
//     set_mie(get_mie() & !(1 << MIE_MEIE_OFFSET));
// }

struct GlobalAllocator {}

static MEMORY_ALLOCATOR: Mutex<allocator::Heap<33>> = Mutex::new(allocator::Heap::new());
static PASS_THROUGH_VIRTIO_MMIO: Mutex<MaybeUninit<VirtioMmio>> =
    Mutex::new(MaybeUninit::<VirtioMmio>::uninit());
static PASS_THROUGH_VIRTIO_BLK_DEVICE: Mutex<MaybeUninit<virtio_blk::VirtioBlk>> =
    Mutex::new(MaybeUninit::<virtio_blk::VirtioBlk>::uninit());
static VIRTUAL_UART_DEVICE: Mutex<Uart> = Mutex::new(Uart::new());

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator {};

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match MEMORY_ALLOCATOR.lock().allocate(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            MEMORY_ALLOCATOR
                .lock()
                .deallocate(NonNull::new_unchecked(ptr), layout);
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn main(argc: usize, argv: *const *const u8) -> usize {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    if argc < 1 {
        panic!("dtb pointer not configured.");
    }

    let fdt_pointer: usize = unsafe {
        match hex_ptr_to_usize(*argv) {
            Ok(address) => address,
            Err(_) => panic!("fdt pointer not configured."),
        }
    };
    println!("booting Halycon...");

    println!("fdt pointer: {:#X}", fdt_pointer);

    let mut host_dt: DeviceTreeInfo<MAX_MEMORY_ENTRIES, MAX_MMIO_ENTRIES> =
        fdt::DeviceTreeInfo::new();

    match host_dt.parse(fdt_pointer as *const u32) {
        Ok(()) => {}
        Err(error) => panic!("{}", error),
    }

    let memory = &host_dt.memory[0];

    unsafe extern "C" {
        static mut _free_area: u8;
    }
    let free_ptr = core::ptr::addr_of!(_free_area) as *const u8 as usize;
    MEMORY_ALLOCATOR
        .lock()
        .init(free_ptr, memory.size - (free_ptr - memory.address));
    println!("[setup] allocator");

    let mut virtio_mmios: Vec<VirtioMmio> = Vec::new();
    for mmio in host_dt.mmio.iter() {
        let virtio_mmio = VirtioMmio::new(mmio.address);
        virtio_mmio.init_default_features();
        virtio_mmios.push(virtio_mmio);
    }

    const BOOTLOADER_MMIO_INDEX: usize = 7;
    const DEVICE_TREE_MMIO_INDEX: usize = 6;
    const PASS_THROUGH_MMIO_INDEX: usize = 5;

    PASS_THROUGH_VIRTIO_MMIO
        .lock()
        .write(virtio_mmios[PASS_THROUGH_MMIO_INDEX]);

    let xlen = get_xlen_from_misa();
    if xlen != 64 {
        println!("this implementation is not 64-bit.");
        return 1;
    }
    println!("[info] XLEN: {}", xlen);

    let misa = get_misa();
    if (misa & (1 << MISA_EXTENSION_H_OFFSET)) == 0 {
        println!("this implementation is not support hypervisor extension.");
        return 1;
    }

    println!("[info] misa: {:#X}", misa);

    // set_mie(get_mie() & (1 << MIE_MEIE_OFFSET));
    // println!("[setup] mie");

    setup_vector();
    println!("[setup] mtvec");
    println!("[setup] stvec");

    // let mut mstatus = get_mstatus();
    // mstatus |= (1 << MSTATUS_TVM_OFFSET) as u64;
    // set_mstatus(mstatus);

    let mut medeleg = get_medeleg();
    medeleg |= (1 << 20) as u64;
    medeleg |= (1 << 12) as u64;
    medeleg |= (1 << 2) as u64;
    medeleg |= (1 << 23) as u64;
    medeleg |= (1 << 22) as u64;
    medeleg |= (1 << 21) as u64;
    medeleg |= (1 << 10) as u64;
    set_medeleg(medeleg);
    println!("[setup] medeleg: {:#X}", medeleg);

    let mut mideleg = get_mideleg();
    mideleg |= MIE_MEIE as u64;
    mideleg |= MIE_VSEIE as u64;
    mideleg |= XIE_SEIE as u64;
    set_mideleg(mideleg);
    println!("[setup] mideleg: {:#X}", mideleg);

    let mut hedeleg = get_hedeleg();
    // hedeleg |= (1 << 12) as u64;
    // hedeleg |= (1 << 7) as u64;
    hedeleg |= (1 << 2) as u64; // Illegal instruction
    set_hedeleg(hedeleg);
    println!("[setup] hedeleg");

    let mut mie = get_mie();
    mie |= MIE_MEIE as u64;
    set_mie(mie);
    println!("[setup] mie");

    let mut sie = get_sie();
    sie |= XIE_SEIE as u64;
    set_sie(sie);
    println!("[setup] sie");

    let mut hie = get_hie();
    hie |= XIE_SEIE as u64;
    set_hie(hie);

    let mut mstatus = get_mstatus();
    mstatus |= MSTATUS_SIE as u64;
    mstatus |= MSTATUS_MIE as u64;
    set_mstatus(mstatus);

    // 仮想マシンの領域のPMPを設定する;
    // let top_address = 0xF0000000 as usize;
    // let bottom_address = 0x80000000 as usize;
    // set_pmp(top_address, bottom_address, true, true, true);
    // println!("[setup] pmp: {:#X} ~ {:#X}", bottom_address, top_address);

    plic::init_plic();

    mmio::ns16550::ns16500_intr_receive_enable();

    set_pmp_all_physical_address(true, true, true);
    println!("[setup] pmpaddr0: {:#X}", get_pmpaddr0());

    unsafe extern "C" {
        static _intr_stack_end: u8;
    }
    unsafe {
        println!(
            "[info] intr stack pointer: {:#X}",
            &_intr_stack_end as *const u8 as usize
        );
    }

    // let stack_address = unsafe { allocate_memory(2, paging::PAGE_SIZE).unwrap() };
    let stack_address = 0x0;
    println!("[info] stack_address: {:#X}", stack_address);

    let vm = vm::create_vm(
        virtio_mmios[BOOTLOADER_MMIO_INDEX],
        virtio_mmios[DEVICE_TREE_MMIO_INDEX],
    );

    println!("switch to guest");
    hs_to_vs(
        vm.get_entry_point() as usize,
        stack_address,
        vm.get_dtb_pointer(),
    )
    // don't return to here
}

fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize, dtb_pointer: usize) -> ! {
    unsafe {
        asm!("
            csrs sstatus, {tmp1}
            csrs hstatus, {tmp2}
            csrw sepc, {entry_point}
            mv sp, {stack_pointer}
            mv a1, {dtb_pointer}
            sret", 
        tmp1 = in(reg) 0x100, // set sstatus.SPP
        tmp2 = in(reg) 0x80, // set hstatus.SPV
        stack_pointer = in(reg) vs_stack_pointer,
        entry_point = in(reg) vs_entry_point,
        dtb_pointer = in(reg) dtb_pointer,
        options(noreturn)
        )
    };
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
