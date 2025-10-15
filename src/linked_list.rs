#[derive(Clone, Copy)]
pub struct LinkedList  {
    head: *mut usize,
}

impl LinkedList {
    pub const fn new() -> Self {
        LinkedList { head: core::ptr::null_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub unsafe fn push(&mut self, value: *mut usize) {
        *value = self.head as usize;
        self.head = value;
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.is_empty() {
            None
        } else {
            let value = self.head;
            self.head = unsafe { *(self.head) as *mut usize };
            Some(value)
        }
    }

    pub fn iter(&self) -> LinkedListIterator {
        
    }
}

pub struct LinkedListIterator {
    current: *mut usize,
}

impl Iterator for LinkedListIteretor {
    type Item = *mut usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let value = self.current;
            self.current = unsafe { *value as *mut usize };
            Some(value)
        }
    }
}
