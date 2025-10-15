use crate::linked_list;
use core::mem;

fn align_up(address: usize, align: usize) -> usize {
    (address + align - 1) & !(align - 1)
}

pub struct Heap<const ORDER: usize> {
    free_list: [linked_list::LinkedList; ORDER],
}

impl<const ORDER: usize> Heap<ORDER> {
    pub const fn new() -> Self {
        Heap {
            free_list: [linked_list::LinkedList::new(); ORDER],
        }
    }

    pub unsafe fn add_to_heap(&mut self, mut address: usize, mut size: usize) {
        // 最低でもusizeでアライン
        address = align_up(address, size_of::<usize>());
        size &= !size_of::<usize>() + 1;
        let end = address + size;

        let mut current_address = address;

        while current_address + size_of::<usize>() <= end {
            // いい感じにpushする
        }
    }
}
