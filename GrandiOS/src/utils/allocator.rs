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
#[derive(Clone)]
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
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
		let mut mem_head = MEM_HEAD.try_lock();
        match mem_head {
			Some(mut head) => {
                if head.data == 0 {
                    // Let's init our linked list for the pages
                    // We want to use as much of our given page as possible
					let refsperpage = self.page/size_of::<PageRef>();
                    let base = self.end-self.page;
                    for i in 0..refsperpage {
                        let tempref = PageRef{
                            page: base-self.page*i, free: i != 0, // We use that one
                            next: if i == refsperpage-1 { 0 } else {
                                base+(i+1)*size_of::<PageRef>()
                            }
                        };
                        write_volatile((base+i*size_of::<PageRef>()) as *mut PageRef, tempref);
                    }
                    head.data = self.end-self.page;
                }

                let mut result = 0;
                let mut address = head.data;
                while result == 0 {
                    let mut current = read_volatile::<PageRef>(address as *mut PageRef);
                    if current.free {
                        current.free = false;
                        result = current.page;
                        write_volatile(address as *mut PageRef, current);
                        break;
                    }
                    //TODO: extend linked list into a new page
                    if current.next == 0 {
                    }
                    address = current.next;
                }
                //TODO: make allocation of more than one page possible
                //let pages = layout.size()/self.page;
                Ok(result as *mut u8)
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
