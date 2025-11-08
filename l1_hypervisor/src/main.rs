#![no_std]
#![no_main]

mod sbi;
mod console;

use core::arch::asm;
use fdt::{DeviceTreeInfo, MemoryEntry};
use string_utils::hex_ptr_to_usize;
use log;

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

    let mut host_dt: DeviceTreeInfo<MAX_MEMORY_ENTRIES, MAX_MMIO_ENTRIES> =
        DeviceTreeInfo::new();

    match host_dt.parse(fdt_pointer as *const u32) {
        Ok(()) => {}
        Err(error) => panic!("{}", error)
    }
    
    let memory: &MemoryEntry = &host_dt.memory[0];
    println!("memory: {:#x}, {:#x}", memory.address, memory.size);

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
