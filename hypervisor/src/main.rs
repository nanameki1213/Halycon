#![feature(riscv_ext_intrinsics)]
#![no_std]
#![no_main]

extern crate alloc;
extern crate lazy_static;

mod aplic;
mod console;
#[cfg(feature = "nested_support")]
mod emulate_csr;
mod memory;
mod paging;
mod plic;
mod sbi;
mod vector;
mod vm;
mod virtual_devices {
    pub mod virtio;
    pub mod serial {
        pub mod ns16550;
    }
}
mod mmio {
    pub mod ns16550;
}

use crate::alloc::string::ToString;
#[cfg(feature = "nested_support")]
use crate::emulate_csr::HypervisorCsr;
use crate::mmio::ns16550::NS16550_ADDR;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use arch::riscv::cpu::*;
use block::mem_blk::MemBlk;
use block::virtio_blk::VirtioBlk;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::marker::PhantomData;
use core::ptr::NonNull;
use fdt::DeviceTreeInfo;
use lazy_static::lazy_static;
use log::{Level, Metadata, Record};
use memory::set_pmp_all_physical_address;
use mmio::ns16550::Uart;
use shmem::ShmRing;
use spin::{Mutex, Once};
use string_utils::hex_ptr_to_usize;
use vector::setup_vector;
use virtio::{VIRTIO_MMIO_DEFAULT_ADDRESS, VirtioMmio};
use vm::VM;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

// guest device
use mmio_core::MmioEntry;

#[macro_export]
macro_rules! bitmask {
    ($high:expr,$low:expr) => {
        ((1 << (($high - $low) + 1)) - 1) << $low
    };
}

const MAX_MEMORY_ENTRIES: usize = 32;
const MAX_MMIO_ENTRIES: usize = 64;

const SHMEM_VIRTUAL_ADDRESS: usize = 0xb0000000;
const SHMEM_SIZE: usize = 0x4000000;

// fn intr_disable() {
//     set_mie(get_mie() & !(1 << MIE_MEIE_OFFSET));
// }

#[derive(Clone, Copy)]
pub struct ShmRingHandle {
    ptr: NonNull<ShmRing>,
    _p: PhantomData<&'static ShmRing>,
}

unsafe impl Send for ShmRingHandle {}
unsafe impl Sync for ShmRingHandle {}

impl ShmRingHandle {
    pub unsafe fn from_base(base: usize) -> Self {
        assert!(
            base % align_of::<ShmRing>() == 0,
            "ShmRing alignment mismatch"
        );
        let ptr = NonNull::new(base as *mut ShmRing).expect("null shm base");
        Self {
            ptr,
            _p: PhantomData,
        }
    }

    #[inline]
    pub fn ring(&self) -> &ShmRing {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub fn ring_mut(&mut self) -> &mut ShmRing {
        unsafe { self.ptr.as_mut() }
    }
}

static LOGGER: SimpleLogger = SimpleLogger;
static MEMORY_ALLOCATOR: Mutex<allocator::Heap<33>> = Mutex::new(allocator::Heap::new());
static VIRTUAL_UART_DEVICE: Mutex<Uart> = Mutex::new(Uart::new());
static CURRENT_VMID: Mutex<usize> = Mutex::new(0);
static SHM_RING: Once<Mutex<ShmRingHandle>> = Once::new();
#[cfg(feature = "nested_support")]
static HOST_HYPERVISOR_CSR: Mutex<HypervisorCsr> = Mutex::new(HypervisorCsr::new());

lazy_static! {
    pub static ref VIRTUAL_MACHINES: Mutex<Vec<VM>> = Mutex::new(Vec::new());
}

pub fn init_shm_ring(base: usize) {
    SHM_RING.call_once(|| {
        let h = unsafe { ShmRingHandle::from_base(base) };
        Mutex::new(h)
    });
}

pub fn with_shm_ring<R>(f: impl FnOnce(&ShmRing) -> R) -> R {
    let m = SHM_RING.get().expect("call init_shm_ring() first");
    let h = *m.lock();
    f(h.ring())
}

pub fn with_shm_ring_mut<R>(f: impl FnOnce(&mut ShmRing) -> R) -> R {
    let m = SHM_RING.get().expect("call init_shm_ring() first");
    let mut h = *m.lock();
    f(h.ring_mut())
}

struct GlobalAllocator {}

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator {};

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match MEMORY_ALLOCATOR.lock().allocate(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match MEMORY_ALLOCATOR.lock().callocate(layout) {
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
    if argc < 1 {
        panic!("dtb pointer not configured.");
    }

    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Trace));

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

    let host_block_device = {
        let mut host_block_device: Option<VirtioBlk> = None;
        // Initialize all virtio mmio device
        for mmio in host_dt.mmio.iter() {
            let virtio_mmio = VirtioMmio::new(mmio.address);
            virtio_mmio.init_default_features();
            virtio_mmios.push(virtio_mmio);

            if mmio.address == 0x10001000 {
                let block_device = VirtioBlk::new(virtio_mmio)
                    .expect("Failed to get block device for hypervisor.");
                host_block_device = Some(block_device);
            }
        }
        host_block_device.expect("No block device for hypervisor.")
    };

    let mut fs = fat32::fat32_init(host_block_device).expect("Failed to init fat32 file system.");

    let vm_img_file_name = "VM.IMG".to_string();
    let file_size = fs
        .get_file_size(&vm_img_file_name)
        .expect("Failed to get file size.");
    let mut buf = vec![0u8; file_size];
    fs.read_file(&vm_img_file_name, &mut buf)
        .expect("Failed to read vm disk image.");
    let mem_block = MemBlk::new(&buf);

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

    let mut hstatus = get_hstatus();
    hstatus |= HSTATUS_VSTR as u64;
    set_hstatus(hstatus);

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

    let mut mmio: Vec<MmioEntry> = Vec::new();

    let serial_entry = MmioEntry::new(
        NS16550_ADDR,
        0x100,
        Box::new(virtual_devices::serial::ns16550::Ns16550),
    );
    mmio.push(serial_entry);

    let virtio_entry = MmioEntry::new(
        VIRTIO_MMIO_DEFAULT_ADDRESS,
        0x1000,
        Box::new(
            virtual_devices::virtio::virtio_mmio::VirtioMmioTransport::new(
                virtual_devices::virtio::virtio_blk::VirtioBlkDevice::new(mem_block),
            ),
        ),
    );
    mmio.push(virtio_entry);

    // let stack_address = unsafe { allocate_memory(2, paging::PAGE_SIZE).unwrap() };
    let stack_address = 0x0;
    println!("[info] stack_address: {:#X}", stack_address);

    let vmid = vm::create_vm(
        fs,
        mmio,
        #[cfg(feature = "nested_support")]
        None,
    );
    let entry_point: usize;
    let dtb_pointer: usize;
    {
        let locked_vm = { VIRTUAL_MACHINES.lock() };
        let vm = &locked_vm[vmid];

        entry_point = vm.get_entry_point() as usize;
        dtb_pointer = vm.get_dtb_pointer();
    }

    println!("switch to guest");
    hs_to_vs(entry_point, stack_address, dtb_pointer)
    // don't return to here
}

fn hs_to_vs(vs_entry_point: usize, vs_stack_pointer: usize, dtb_pointer: usize) -> ! {
    let mut hstatus = get_hstatus();
    hstatus |= HSTATUS_SPV as u64;
    set_hstatus(hstatus);
    let mut sstatus = get_sstatus();
    sstatus |= SSTATUS_SPP as u64;
    set_sstatus(sstatus);

    unsafe {
        asm!("
            csrw sepc, {entry_point}
            mv sp, {stack_pointer}
            mv a1, {dtb_pointer}
            sret", 
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
