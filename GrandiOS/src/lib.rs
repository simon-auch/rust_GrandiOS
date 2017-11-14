#![no_std]
#![feature(lang_items)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
#![feature(range_contains)]
#![feature(slice_concat_ext)]
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
    pub mod parser;
    pub mod shell;
}
mod commands{
    pub mod logo;
    pub mod cat;
    pub mod test;
    pub mod edit;
    pub mod cowsay;
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
    //commands::logo::draw();
    //make interupt table writable
    let mut mc = unsafe { MemoryController::new(MC_BASE_ADRESS) } ;
    mc.remap();
    /*
    //enable interrupts
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.set_handler(1, &(default_handler_2 as fn())); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.set_priority(1, 4);
    ic.set_sourcetype(1, 2);//positive edge triggered
    ic.enable();
    DEBUG_UNIT.interrupt_set_rxrdy(true);
    */
    /* //Fasst der richtige code zum anschalten der interrupts (IRQ + FIQ), von dem Register CPSR müssen jeweils bit 7 und 6 auf 0 gesetzt werden, damit die interrupts aufgerufen werden.
    unsafe{
        asm!(
            "MSR CPSR_c, 0b0000000":
            :
            :
            :
        )
    }
    */
    println!("Memory location of default handler: {:p}", &(default_handler_2 as fn()));
    println!("What is written to the aic; {:x}", ((&(default_handler_2 as fn())) as *const _) as u32);
    loop{}
    utils::shell::run();
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


fn default_handler_2(){
    let mut DEBUG_UNIT_b = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(DEBUG_UNIT_b, "hi 2\n");
    loop{}
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
