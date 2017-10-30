// Basis: https://doc.rust-lang.org/nightly/unstable-book/language-features/global-allocator.html

extern crate alloc;

use alloc::heap::{Alloc, Layout, AllocErr};

pub struct Allocator;

unsafe impl<'a> Alloc for &'a Allocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        Ok(0x23_fff_fff as *mut u8)
    }
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
    }
}
