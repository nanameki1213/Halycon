#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

extern crate alloc;

use alloc::alloc::{Allocator, Global, Layout};

const PAGE_SIZE: usize = 0x1000;

pub fn allocate_pages_in<A: Allocator>(num_of_pages: usize, align: usize, allocator: A) -> *mut u8 {
    let layout = unsafe { Layout::from_size_align_unchecked(num_of_pages * PAGE_SIZE, align) };

    match allocator.allocate(layout) {
        Ok(ptr) => ptr.as_mut_ptr(),
        Err(_) => core::ptr::null_mut(),
    }
}

pub fn allocate_pages(num_of_pages: usize, align: usize) -> *mut u8 {
    allocate_pages_in(num_of_pages, align, Global)
}

pub fn callocate_pages_in<A: Allocator>(
    num_of_pages: usize,
    align: usize,
    allocator: A,
) -> *mut u8 {
    let layout = unsafe { Layout::from_size_align_unchecked(num_of_pages * PAGE_SIZE, align) };

    match allocator.allocate_zeroed(layout) {
        Ok(ptr) => ptr.as_mut_ptr(),
        Err(_) => core::ptr::null_mut(),
    }
}

pub fn callocate_pages(num_of_pages: usize, align: usize) -> *mut u8 {
    callocate_pages_in(num_of_pages, align, Global)
}
