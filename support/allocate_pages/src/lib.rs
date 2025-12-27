#![no_std]
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
