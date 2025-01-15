use core::arch::riscv64;
use core::usize;

use crate::allocate_memory;
use crate::paging;
use crate::cpu::*;
use crate::loader;
use crate::println;

static mut VMID: usize = 0;

pub struct VM {
    vmid: usize,
    ram_virtual_base_address: usize,
    ram_physical_base_address: usize,
    ram_size: usize,
}

impl VM {
    fn new(
        ram_virtual_base_address: usize,
        ram_physical_base_address: usize,
        ram_size: usize
    ) -> *mut Self {
        let vm = unsafe {
            &mut *(allocate_memory(1, paging::PAGE_SIZE).unwrap() as *mut VM)
        };

        vm.ram_virtual_base_address = ram_virtual_base_address;
        vm.ram_physical_base_address = ram_physical_base_address;
        vm.ram_size = ram_size;
        unsafe {
            vm.vmid = VMID;
            VMID += 1;
        }

        vm
    }

    pub fn get_entry_point(&mut self) -> usize {
        self.ram_physical_base_address
    }
}

pub fn create_vm() -> *mut VM {
    const RAM_VIRTUAL_BASE: usize = 0x80000000;
    const RAM_SIZE: usize = 0x10000000;

    let ram_physical_base_address = unsafe {
        allocate_memory(RAM_SIZE / paging::PAGE_SIZE, paging::PAGE_SIZE).unwrap()
    };

    let vm = VM::new(RAM_VIRTUAL_BASE, ram_physical_base_address, RAM_SIZE);

    let table_address = paging::map_address_stage2(ram_physical_base_address, RAM_VIRTUAL_BASE, RAM_SIZE, paging::DEFAULT_TABLE_LEVEL, true, true, true).expect("Failed to mapping");
    println!("[info] table address: {:#X}", table_address);
    let mut hgatp = match paging::DEFAULT_TABLE_LEVEL {
        3 => 0b1000 << 60,
        4 => 0b1001 << 60,
        5 => 0b1010 << 60,
        _ => unreachable!(),
    };
    hgatp |= (table_address >> 12) & SATP_PPN_MASK;
    set_hgatp(hgatp as u64);
    
    println!("[info] vm virtual address: {:#X}", RAM_VIRTUAL_BASE);
    println!("[info] vm physical address: {:#X}", ram_physical_base_address);

    println!("[info] loading u-boot...");
    loader::load_bootloader(ram_physical_base_address);

    vm
}
