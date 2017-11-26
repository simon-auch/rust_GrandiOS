use alloc::vec::Vec;

pub struct Ring<T> {
    head: usize,
    tail: usize,
    data: Vec<T>,
}

impl <T> Ring<T> {
    pub fn new(size: usize) -> Ring<T> {
        Ring{head: 0, tail: 0, data: Vec::with_capacity(size)}
    }
    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }
    pub fn push(&mut self, elem: T) {
        self.data[self.tail] = elem;
        self.tail += 1;
        if self.tail == self.data.capacity() { self.tail = 0; }
    }
    pub fn pop(&mut self) -> Option<&T> {
        if self.is_empty() { return None; }
        let res = &self.data[self.head];
        self.head += 1;
        if self.head == self.data.capacity() { self.head = 0; }
        Some(res)
    }
}
