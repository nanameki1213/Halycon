extern crate alloc;

use crate::VIRTUAL_MACHINES;
#[cfg(feature = "nested_support")]
use crate::emulate_csr::HypervisorCsr;
use crate::paging;
use crate::println;
use alloc::string::ToString;
use alloc::vec::Vec;
use allocate_pages::allocate_pages;
use arch::riscv::cpu::*;
use block::BlockDevice;
use core::arch::riscv64;
use fat32::Fat32;
use mmio_core::MmioEntry;

#[cfg(feature = "nested_support")]
#[derive(Clone, Copy, Debug)]
pub struct HypervisorContext {
    pub csr: HypervisorCsr,
    pub vmid: usize,
}

#[cfg(feature = "nested_support")]
impl HypervisorContext {
    pub const fn new() -> Self {
        HypervisorContext {
            csr: HypervisorCsr::new(),
            vmid: 0,
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
    #[cfg(feature = "nested_support")]
    pub parent_vmid: Option<usize>,
    #[cfg(feature = "nested_support")]
    pub hypervisor: Option<HypervisorContext>,
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
        #[cfg(feature = "nested_support")] parent_vmid: Option<usize>,
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
            #[cfg(feature = "nested_support")]
            parent_vmid,
            #[cfg(feature = "nested_support")]
            hypervisor: None,
        }
    }

    pub fn get_entry_point(&self) -> usize {
        self.entry_point
    }

    pub fn get_dtb_pointer(&self) -> usize {
        self.dtb_pointer
    }
}

pub fn create_vm<T: BlockDevice>(
    mut fs: Fat32<T>,
    mmio: Vec<MmioEntry>,
    #[cfg(feature = "nested_support")] parent_vmid: Option<usize>,
) -> usize {
    const RAM_VIRTUAL_BASE: usize = 0x80000000;
    const RAM_SIZE: usize = 0x20000000;

    let ram_physical_base_address = allocate_pages(RAM_SIZE / paging::PAGE_SIZE, paging::PAGE_SIZE);
    if ram_physical_base_address.is_null() {
        println!("Out of memory");
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
    unsafe {
        riscv64::hfence_gvma_all();
        riscv64::hfence_vvma_all();
        riscv64::sfence_vma_all();
    }

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
    fs.read_file(&bios_file_name, bootloader_entry_point as *mut u8)
        .expect("Failed to read file");

    let dtb_file_name = "VIRT.DTB".to_string();
    println!("[info] loading {}...", dtb_file_name);
    fs.read_file(&dtb_file_name, dtb_pointer as *mut u8)
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
        #[cfg(feature = "nested_support")]
        parent_vmid,
    );
    locked_vms.push(vm);

    vmid
}

#[cfg(feature = "nested_support")]
pub fn create_l2_vm(parent_vmid: usize, vms: &mut Vec<VM>) -> usize {
    let new_vmid = vms.len();
    let l2_vm = VM::new(new_vmid, 0, 0, 0, 0, 0, 0, Vec::new(), Some(parent_vmid));
    vms.push(l2_vm);

    new_vmid
}
