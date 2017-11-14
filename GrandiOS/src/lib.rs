#![no_std]
#![feature(lang_items)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
#![feature(range_contains)]
//disable some warnings
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_unsafe)]
#![allow(unused_mut)]
#![allow(dead_code)]
//alloc needs lots of features
#![feature(alloc, global_allocator, allocator_api, heap_api)]
#![feature(compiler_builtins_lib)]
//Include other parts of the kernal

#[macro_use]
mod driver{
	#[macro_use]
	pub mod serial;
	pub mod led;
	pub mod memory_controller;
	pub mod interrupts;

	pub use serial::*;
	pub use led::*;
	pub use memory_controller::*;
	pub use interrupts::*;
}
mod utils{
	pub mod spinlock;
	pub mod allocator;
    pub mod thread;
    pub mod shell;
}
mod commands{
    pub mod logo;
    pub mod cat;
    pub mod test;
}
use driver::*;

#[global_allocator]
static GLOBAL: utils::allocator::Allocator = utils::allocator::Allocator::new( 0x22000000, 1<<10);
#[macro_use]
extern crate alloc;
extern crate compiler_builtins;
extern crate rlibc;

//#[no_mangle]
//keep the function name so we can call it from assembler
//pub extern
//make the function use the standard C calling convention
#[no_mangle]
#[naked]
pub extern fn _start() {
    //TODO: Initialise the stack pointers for all modes (system, abort, irq, fiq, etc)
    //Initialise the DebugUnit
    DEBUG_UNIT.reset();
    DEBUG_UNIT.enable();
    //commands::logo::draw();
    //make interupt table writable
    let mut mc = unsafe { MemoryController::new(MC_BASE_ADRESS) } ;
    mc.remap();
    utils::shell::run();
}


// These functions and traits are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
pub fn panic_fmt() -> ! { loop {} }

// We need this to remove a linking error for the allocator
#[no_mangle]
pub unsafe fn __aeabi_unwind_cpp_pr0() { loop {} }
