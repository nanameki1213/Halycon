#![no_std]
#![no_main]

mod console;
mod memory;
mod paging;

use arch::riscv::sbi;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use fdt::{DeviceTreeInfo, MemoryEntry};
use log;
use spin::Mutex;
use string_utils::hex_ptr_to_usize;

pub struct SbiConsoleLogger;

impl log::Log for SbiConsoleLogger {
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

static LOGGER: SbiConsoleLogger = SbiConsoleLogger;

struct GlobalAllocator {}

static MEMORY_ALLOCATOR: Mutex<allocator::Heap<33>> = Mutex::new(allocator::Heap::new());

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
        MEMORY_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), layout);
    }
}

const MAX_MEMORY_ENTRIES: usize = 32;
const MAX_MMIO_ENTRIES: usize = 64;

#[unsafe(no_mangle)]
extern "C" fn main(argc: usize, argv: *const *const u8) {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);

    if argc < 1 {
        panic!("fdt pointer isn't configured");
    }

    let fdt_pointer: usize = unsafe {
        match hex_ptr_to_usize(*argv) {
            Ok(address) => address,
            Err(_) => panic!("hex format is wrong"),
        }
    };
    println!("booting hypervisor...");

    println!("fdt pointer: {:#x}", fdt_pointer);

    let mut host_dt: DeviceTreeInfo<MAX_MEMORY_ENTRIES, MAX_MMIO_ENTRIES> = DeviceTreeInfo::new();

    match host_dt.parse(fdt_pointer as *const u32) {
        Ok(()) => {}
        Err(error) => panic!("{}", error),
    }

    let memory: &MemoryEntry = &host_dt.memory[0];
    println!("memory: {:#x}, {:#x}", memory.address, memory.size);

    unsafe extern "C" {
        static mut _free_area: u8;
    }
    let free_ptr = core::ptr::addr_of!(_free_area) as *const u8 as usize;
    unsafe {
        MEMORY_ALLOCATOR
            .lock()
            .init(free_ptr, memory.size - (free_ptr - memory.address));
    }
    println!("[setup] allocator");


    paging::map_address_stage2(0x80000000, virtual_address, map_size, table_level, is_readable, is_writable, is_executable)

    halt_loop();
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("\n\nPanic; {}", info);
    halt_loop();
}
