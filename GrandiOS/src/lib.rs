#![no_std]
#![feature(lang_items)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
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
}
mod commands{
    pub mod logo;
    pub mod cat;
    pub mod test;
}
use driver::*;
use commands::*;

#[global_allocator]
static GLOBAL: utils::allocator::Allocator = utils::allocator::Allocator::new( 0x22000000, 1<<10);
#[macro_use]
extern crate alloc;
extern crate compiler_builtins;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
extern crate rlibc;

//#[no_mangle]
//keep the function name so we can call it from assembler
//pub extern
//make the function use the standard C calling convention
#[no_mangle]
#[naked]
pub extern fn _start() {
    let commands = vec![("logo", logo::exec as fn(Vec<&str>)),
                        ("test", test::exec as fn(Vec<&str>)),
                        ("cat", cat::exec as fn(Vec<&str>))];
	//Initialise the LED's
	let mut led_yellow = unsafe { driver::led::PIO::new(driver::led::PIO_LED_YELLOW) };
	let mut led_red    = unsafe { driver::led::PIO::new(driver::led::PIO_LED_RED)    };
	let mut led_green  = unsafe { driver::led::PIO::new(driver::led::PIO_LED_GREEN)  };
	led_yellow.off();
	led_red.off();
	led_green.off();
	//Initialise the DebugUnit
	DEBUG_UNIT.reset();
	DEBUG_UNIT.enable();
    //logo::draw();
    //make interupt table writable
    let mut mc = unsafe { MemoryController::new(MC_BASE_ADRESS) } ;
    mc.remap();
    //enable interrupts
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.enable();
    for i in 0..32{
		ic.set_handler(i, &(default_handler_2 as fn()));
	}
	DEBUG_UNIT.interrupt_set_rxrdy(true);
    loop {
        let input = String::from_utf8(echo_readln!("> ")).expect("Found invalid UTF-8");
        let mut arguments: Vec<&str> = input.split(' ').collect();
        let command = arguments.remove(0);
        let mut found = false;
        for &(c, m) in commands.iter() {
            if command == c {
                found = true;
                m(arguments);
                break;
            }
        }
        if !found {
            println!("Unknown command!");
        }
    }
}

#[no_mangle]
#[naked]
extern fn testing(){
	unsafe {
		asm!(
			"LDR PC,[PC, # -0xF20]" :
			:/*outputs*/
			:/*inputs*/
			:/*clobbers*/
			/*options*/
		);
	}
}

#[allow(dead_code)]
#[link_section = ".rodata.interrupts"]
static INTERRUPTS: [extern "C" fn(); 7] = [default_handler; 7];
#[no_mangle]
#[naked]
extern "C" fn default_handler() {
    let mut DEBUG_UNIT_b = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(DEBUG_UNIT_b, "hi");
}
#[naked]
fn default_handler_2(){
	let mut DEBUG_UNIT_b = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(DEBUG_UNIT_b, "hi 2\n");
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
