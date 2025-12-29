#![cfg_attr(not(test), no_std)]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]

extern crate alloc;

use core::ptr::NonNull;

use alloc::alloc::{AllocError, Allocator, Global, Layout};

pub const PAGE_SIZE: usize = 0x1000;

pub struct Pages<A: Allocator = Global> {
    ptr: NonNull<u8>,
    count: usize,
    align: usize,
    alloc: A,
}

impl Pages<Global> {
    pub fn new(count: usize, align: usize) -> Result<Self, AllocError> {
        Self::new_in(count, align, Global)
    }

    pub fn new_zeroed(count: usize, align: usize) -> Result<Self, AllocError> {
        Self::new_zeroed_in(count, align, Global)
    }
}

impl<A: Allocator> Pages<A> {
    pub fn new_in(count: usize, align: usize, alloc: A) -> Result<Self, AllocError> {
        if count == 0 {
            return Err(AllocError);
        }

        let layout = unsafe { Layout::from_size_align_unchecked(count * PAGE_SIZE, align) };

        match alloc.allocate(layout) {
            Ok(ptr) => {
                let pages = Pages {
                    ptr: ptr.cast::<u8>(),
                    count,
                    align,
                    alloc,
                };
                Ok(pages)
            }
            Err(err) => Err(err),
        }
    }

    pub fn new_zeroed_in(count: usize, align: usize, alloc: A) -> Result<Self, AllocError> {
        if count == 0 {
            return Err(AllocError);
        }

        let layout = unsafe { Layout::from_size_align_unchecked(count * PAGE_SIZE, align) };

        match alloc.allocate_zeroed(layout) {
            Ok(ptr) => {
                let pages = Pages {
                    ptr: ptr.cast::<u8>(),
                    count,
                    align,
                    alloc,
                };
                Ok(pages)
            }
            Err(err) => Err(err),
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe { self.ptr.as_mut() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.count * PAGE_SIZE) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.count * PAGE_SIZE) }
    }
}

impl<A: Allocator> Drop for Pages<A> {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.count * PAGE_SIZE, self.align);
            self.alloc.deallocate(self.ptr, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_allocation() {
        let count = 3;
        let align = PAGE_SIZE;
        let mut pages = Pages::new(count, align).expect("Allocation failed");

        let slice = pages.as_bytes();
        assert_eq!(slice.len(), count * PAGE_SIZE);

        let slice_mut = pages.as_bytes_mut();
        slice_mut[0] = 0xAA;
        slice_mut[slice_mut.len() - 1] = 0xBB;

        assert_eq!(pages.as_bytes()[0], 0xAA);
        assert_eq!(pages.as_bytes()[count * PAGE_SIZE - 1], 0xBB);
    }

    #[test]
    fn test_new_zeroed() {
        let count = 2;
        let align = 16;
        let mut pages = Pages::new_zeroed(count, align).expect("Allocation failed");

        let slice = pages.as_bytes();
        for &byte in slice {
            assert_eq!(byte, 0, "Memory was not zeroed");
        }

        pages.as_bytes_mut()[0] = 1;
        assert_eq!(pages.as_bytes()[0], 1);
    }

    #[test]
    fn test_alignment() {
        let count = 1;
        let align = 8192;
        let pages = Pages::new(count, align).expect("Allocation failed");

        let addr = pages.as_ptr() as usize;
        assert_eq!(
            addr % align,
            0,
            "Address {:x} is not aligned to {}",
            addr,
            align
        );
    }

    #[test]
    fn test_zero_count_error() {
        let result = Pages::new(0, PAGE_SIZE);
        assert!(result.is_err());
    }
}
