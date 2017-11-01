use alloc::heap::{Alloc, Layout, AllocErr};
use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::read_volatile;
use core::ptr::write_volatile;
// We need to have something static but mutable
use utils::spinlock;

pub struct Allocator {
    end: usize,
    size: usize,
    page: usize
}

#[repr(C)]
#[derive(Clone,Copy)]
struct PageRef {
    page: usize,
    free: bool,
    next: usize
}

struct MemHead {
    data: usize
}

static MEM_HEAD: spinlock::Spinlock<MemHead> = spinlock::Spinlock::new(MemHead{data: 0});

impl Allocator {
	pub const fn new() -> Self {
        Allocator { end: 0x23ffffff, size: 1<<20, page: 1<<10 }
    }
    pub unsafe fn prepare(&self, base: usize) {
        // Let's init our linked list for the pages
        // We want to use as much of our given page as possible
        let refsperpage = self.page/size_of::<PageRef>();
        for i in 0..refsperpage {
            let tempref = PageRef{
                page: base-self.page*i, free: i != 0, // We use that one
                next: if i == refsperpage-1 { 0 } else {
                    base+(i+1)*size_of::<PageRef>()
                }
            };
            write_volatile((base+i*size_of::<PageRef>()) as *mut PageRef, tempref);
        }
    }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
		let mut head = MEM_HEAD.lock();
        if head.data == 0 {
            self.prepare(self.end-self.page);
            head.data = self.end-self.page;
        }
        let pages = layout.size()/self.page+(if layout.size()%self.page==0 {0} else {1});
        let mut address = head.data;
        let mut first = 0;
        let mut count = 0;
        loop {
            let mut current = read_volatile::<PageRef>(address as *mut PageRef);
            if current.free {
                if count == 0 { first = address; }
                count += 1;
                if count == pages { break; }
            } else {
                count = 0;
            }
            // Extend linked list into a new page
            if current.next == 0 {
                let base = current.page-self.page;
                self.prepare(base);
                current.next = base;
                write_volatile(address as *mut PageRef, current);
            }
            address = current.next;
        }
        address = first;
        for i in 0..pages {
            let mut current = read_volatile::<PageRef>(address as *mut PageRef);
            current.free = false;
            write_volatile(address as *mut PageRef, current);
            address = current.next;
        }
        let current = read_volatile::<PageRef>(first as *mut PageRef);
        Ok(current.page as *mut u8)
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    }
}
