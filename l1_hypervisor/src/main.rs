#![no_std]
#![no_main]

mod sbi;
mod console;

use core::arch::asm;

#[unsafe(no_mangle)]
extern "C" fn main(argc: usize, argv: *const *const u8) {
    println!("Hello from hypervisor!");

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
