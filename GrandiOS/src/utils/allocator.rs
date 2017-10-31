extern crate alloc;

use alloc::heap::{Alloc, Layout, AllocErr};
use alloc::boxed::Box;
// We need to have something static but mutable
use utils::spinlock;

pub struct Allocator {
    end: usize,
    size: usize,
    page: usize
}

#[repr(C)]
struct PageRef {
    start: usize,
    count: u8,
    next: u8
}

struct MemHead {
    data: Option<Box<PageRef>>
}

static MEM_HEAD: spinlock::Spinlock<MemHead> = spinlock::Spinlock::new(MemHead{data: None});

impl Allocator {
	pub const fn new() -> Self { Allocator { end: 0x23ffffff, size: 2*2^20, page: 2*2^10 } }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
		let mut mem_head = MEM_HEAD.try_lock();
		match mem_head{
			Some(mut head) => {
                if head.data.is_none() {
                    head.data = Some(Box::new(PageRef{start: self.end-self.page, count:1, next: 0}));
                }
                //TODO: actually use the fucking linked list
                let pages = 2;
                Ok((self.end-pages*self.page) as *mut u8)
            },
			None => {
                // We failed to get the lock because we went here from inside
                // That means we want to get some space for the memory head
                // This will always be the very first page available
                Ok((self.end-self.page) as *mut u8)
            }
		}
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    }
}
