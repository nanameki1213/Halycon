#![no_std]
#![no_main]

use core::arch::asm;

#[unsafe(no_mangle)]
extern "C" fn main(argc: usize, argv: *const *const u8) {
    halt_loop();
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    halt_loop();
}
