use driver::serial::*;
use alloc::heap::{Alloc, Layout, AllocErr};
use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
// We need to have something static but mutable
use utils::spinlock;

pub struct Allocator {
    start: usize,
    end: usize
}

#[repr(C)]
#[derive(Clone,Copy)]
struct PageRef {
    start: usize,
    size: usize,
    next: usize
}

struct MemHead {
    data: usize
}

static MEM_HEAD: spinlock::Spinlock<MemHead> = spinlock::Spinlock::new(MemHead{data: 0});

impl Allocator {
	pub const fn new(base: usize, end: usize) -> Self {
        Allocator { start: base, end: end }
    }
    pub unsafe fn new_empty(&self, base: usize, next: usize) {
        let s = base+size_of::<PageRef>();
        let tempref = PageRef{
            start: s, next: next,
            size: if next == 0 { self.end } else { next }-s 
        };
        write_volatile(base as *mut PageRef, tempref);
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
		let mut head = MEM_HEAD.lock();
        if head.data == 0 {
            self.new_empty(self.start, 0);
            head.data = self.start;
        }
        let mut address = head.data;
        loop {
            let mut current = read_volatile::<PageRef>(address as *mut PageRef);
            if current.size >= layout.size()+size_of::<PageRef>() {
                let base = current.start+layout.size();
                current.size = 0;
                self.new_empty(base, current.next);
                current.next = base;
                write_volatile(address as *mut PageRef, current);
                return Ok(current.start as *mut u8);
            }
            if current.next == 0 {
                return Err(AllocErr::Exhausted {request: layout});
            }
            address = current.next;
        }
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
		let head = MEM_HEAD.lock();
        let mut address = head.data;
        let mut aprev = 0;
        /*
        println!("Want to free {:p}", ptr);
        print!("We have ");
        loop {
            if address == 0 { break; }
            let mut current = read_volatile::<PageRef>(address as *mut PageRef);
            print!("{:x} ", current.start);
            address = current.next;
        }
        println!("");
        address = head.data;
        */
        loop {
            if address == 0 { loop {} }
            let mut current = read_volatile::<PageRef>(address as *mut PageRef);
            if current.start == ptr as usize {
                current.size = layout.size();
                let mut prev = read_volatile::<PageRef>(aprev as *mut PageRef);
                let next = read_volatile::<PageRef>(current.next as *mut PageRef);
                if next.size != 0 { //merge
                    current.size += next.size+size_of::<PageRef>();
                    current.next = next.next;
                }
                if aprev != 0 && prev.size != 0 { //merge
                    prev.size += current.size+size_of::<PageRef>();
                    prev.next = current.next;
                    write_volatile(aprev as *mut PageRef, prev);
                }
                write_volatile(address as *mut PageRef, current);
                return;
            }
            aprev = address;
            address = current.next;
        }
    }
}
