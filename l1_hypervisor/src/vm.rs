extern crate alloc;

use crate::VIRTUAL_MACHINES;
use crate::paging;
use crate::println;
use alloc::string::ToString;
use alloc::vec::Vec;
use allocate_pages::allocate_pages;
use arch::riscv::cpu::*;
use block::BlockDevice;
use core::slice;
use fat32::Fat32;
use mmio_core::MmioEntry;

#[derive(Debug, Clone, Copy)]
pub struct Csr {
    pub stvec: u64,
    pub sepc: u64,
    pub sstatus: u64,
    pub scause: u64,
    pub stval: u64,
    pub satp: u64,
}

impl Csr {
    pub const fn new() -> Self {
        Csr {
            stvec: 0,
            sepc: 0,
            sstatus: 0,
            scause: 0,
            stval: 0,
            satp: 0,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct VM {
    pub vmid: usize,
    pub page_table_address: usize,
    pub ram_virtual_base_address: usize,
    pub ram_physical_base_address: usize,
    pub ram_size: usize,
    pub entry_point: usize,
    pub dtb_pointer: usize,
    pub mmio: Vec<MmioEntry>,
    pub vcsr: Csr,
}

impl VM {
    pub const fn new(
        vmid: usize,
        page_table_address: usize,
        ram_virtual_base_address: usize,
        ram_physical_base_address: usize,
        ram_size: usize,
        entry_point: usize,
        dtb_pointer: usize,
        mmio: Vec<MmioEntry>,
    ) -> Self {
        VM {
            vmid,
            page_table_address,
            ram_virtual_base_address,
            ram_physical_base_address,
            ram_size,
            entry_point,
            dtb_pointer,
            mmio,
            vcsr: Csr::new(),
        }
    }

    pub fn get_entry_point(&self) -> usize {
        self.entry_point
    }

    pub fn get_dtb_pointer(&self) -> usize {
        self.dtb_pointer
    }
}

pub fn create_vm<T: BlockDevice>(mut fs: Fat32<T>, mmio: Vec<MmioEntry>) -> usize {
    const RAM_VIRTUAL_BASE: usize = 0x80000000;
    const RAM_SIZE: usize = 0x10000000;

    let ram_physical_base_address = allocate_pages(RAM_SIZE / paging::PAGE_SIZE, paging::PAGE_SIZE);
    if ram_physical_base_address.is_null() {
        panic!("Out of memory");
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
    println!(
        "[info] vm physical address: {:#X}",
        ram_physical_base_address as usize
    );

    let files = fs
        .list_root_files()
        .expect("Failed to get list of root directory files.");
    println!("files:");
    for file in files {
        println!("{:?}", file);
    }

    let virtual_entry_point = 0x80200000;
    let bootloader_entry_point = paging::resolve_address_stage2(virtual_entry_point).unwrap();
    println!(
        "[info] vm entry point physical address: {:#X}",
        bootloader_entry_point
    );

    let virtual_dtb_pointer = RAM_VIRTUAL_BASE;
    let dtb_pointer = paging::resolve_address_stage2(virtual_dtb_pointer).unwrap();

    let bios_file_name = "U-BOOT.BIN".to_string();
    println!("[info] loading {}...", bios_file_name);
    let size = fs
        .get_file_size(&bios_file_name)
        .expect("Failed to get file size.");
    let buf = unsafe { slice::from_raw_parts_mut(bootloader_entry_point as *mut u8, size) };
    fs.read_file(&bios_file_name, buf)
        .expect("Failed to read file");

    let dtb_file_name = "VIRT.DTB".to_string();
    println!("[info] loading {}...", dtb_file_name);
    let size = fs
        .get_file_size(&dtb_file_name)
        .expect("Failed to get file size.");
    let buf = unsafe { slice::from_raw_parts_mut(dtb_pointer as *mut u8, size) };
    fs.read_file(&dtb_file_name, buf)
        .expect("Failed to read file");

    let mut locked_vms = VIRTUAL_MACHINES.lock();
    let vmid = locked_vms.len();
    let vm = VM::new(
        vmid,
        table_address,
        RAM_VIRTUAL_BASE,
        ram_physical_base_address as usize,
        RAM_SIZE,
        virtual_entry_point,
        virtual_dtb_pointer,
        mmio,
    );
    locked_vms.push(vm);

    vmid
}
