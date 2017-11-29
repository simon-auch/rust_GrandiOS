use swi;
use alloc::heap::{Alloc, Layout, AllocErr};

pub struct Allocator {}

impl Allocator {
	pub const fn new() -> Self { Allocator{}  }
}

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        let input      = swi::useralloc::Input{l:layout};
        let mut output = swi::useralloc::Output{r:None};
        swi::useralloc::call(& input, &mut output);
        output.r.unwrap()
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let input      = swi::userdealloc::Input{p:ptr, l:layout};
        let mut output = swi::userdealloc::Output{};
        swi::userdealloc::call(& input, &mut output);
    }
}
