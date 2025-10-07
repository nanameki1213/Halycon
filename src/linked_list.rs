pub struct LinkedList  {
    head: *mut usize,
}

impl LinkedList {
    pub fn new() -> Self {
        LinkedList { head: std::ptr::null_mut() }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub fn push(&mut self, )
}
