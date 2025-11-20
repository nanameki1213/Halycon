#![no_std]
#![no_main]

mod console;
mod memory;
mod paging;

use arch::riscv::cpu::*;
use arch::riscv::sbi;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use fdt::{DeviceTreeInfo, MemoryEntry};
use spin::Mutex;
use string_utils::hex_ptr_to_usize;

use crate::memory::allocate_pages;

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
        unsafe {
            MEMORY_ALLOCATOR
                .lock()
                .deallocate(core::ptr::NonNull::new_unchecked(ptr), layout);
        }
    }
}

const MAX_MEMORY_ENTRIES: usize = 32;
const MAX_MMIO_ENTRIES: usize = 64;

#[unsafe(no_mangle)]
extern "C" fn main(argc: usize, argv: *const *const u8) {
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
    MEMORY_ALLOCATOR
        .lock()
        .init(free_ptr, memory.size - (free_ptr - memory.address));
    println!("[setup] allocator");

    let mut hedeleg = get_hedeleg();
    hedeleg |= (1 << 2) as u64; // Illegal instruction
    set_hedeleg(hedeleg);
    println!("[setup] hedeleg");

    let mut hie = get_hie();
    hie |= XIE_SEIE as u64;
    set_hie(hie);
    println!("[setup] hie");

    let sstatus = get_sstatus();
    println!("[info] sstatus: {:#x}", sstatus as usize);

    const RAM_VIRTUAL_BASE: usize = 0x80000000;
    const RAM_SIZE: usize = 0x8000000;

    let ram_physical_base_address = allocate_pages(RAM_SIZE / paging::PAGE_SIZE, paging::PAGE_SIZE);
    if ram_physical_base_address.is_null() {
        println!("out of memory.");
        panic!();
    }
    let table_address = paging::map_address_stage2(
        ram_physical_base_address as usize,
        RAM_VIRTUAL_BASE,
        RAM_SIZE,
        paging::DEFAULT_TABLE_LEVEL,
        true,
        true,
        true,
    )
    .expect("Failed to mapping");
    println!("[info] table_address");
    let mut hgatp = match paging::DEFAULT_TABLE_LEVEL {
        3 => 0b1000 << 60,
        4 => 0b1001 << 60,
        5 => 0b1010 << 60,
        _ => unreachable!(),
    };
    hgatp |= (table_address >> 12) & SATP_PPN_MASK;
    set_hgatp(hgatp as u64);

    println!("[info] vm virtual address: {:#X}", RAM_VIRTUAL_BASE);
    println!(
        "[info] vm physical address: {:#X}",
        ram_physical_base_address as usize
    );

    let stack_size = 0x2000;
    let stack_memory = allocate_pages(stack_size / paging::PAGE_SIZE, paging::PAGE_SIZE);
    if stack_memory.is_null() {
        println!("Failed to allocate memory for VM stack.");
        panic!();
    }
    let stack_pointer = stack_memory as usize + stack_size;

    let virtual_entry_point = 0x80200000;
    let physical_entry_point = 0x0;
    println!(
        "[info] vm entry point physical address: {:#X}",
        physical_entry_point
    );

    println!("switch to guest");
    hs_to_vs(virtual_entry_point, stack_pointer, 0x0)
}

pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("wfi") };
    }
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
