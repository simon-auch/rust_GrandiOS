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
    pub mod registers;
}
mod commands{
    pub mod logo;
    pub mod cat;
    pub mod test;
    pub mod edit;
    pub mod cowsay;
    pub mod math;
    pub mod higher;
}
use driver::*;

#[global_allocator]
static GLOBAL: utils::allocator::Allocator = utils::allocator::Allocator::new(0x22000000, 0x23ffffff);
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
    //initialisiert register, stack pointer und remaped speicher
    init();
    //Initialise the DebugUnit
    DEBUG_UNIT.reset();
    DEBUG_UNIT.enable();
    //commands::logo::draw();
    utils::shell::run();
}

#[inline(always)]
#[naked]
fn init(){
    //make interupt table writable
    let mut mc = unsafe { MemoryController::new(MC_BASE_ADRESS) } ;
    mc.remap();
    //initialise the stack pointers for all modes.
    //each stack gets around 1kbyte, except the fiq which has a bit less (vector table+ jump addresses) and the system/user stack which has 11kbyte
    unsafe{asm!("
        mrs     r0, CPSR		//auslaesen vom status register
        bic     r0, r0, #0x1F	//set all mode bits to zero
        orr     r1, r0, #0x11	//ARM_MODE_FIQ
        msr     CPSR, r1
        mov     sp, #0x400	//set stack pointer for fiq mode
        orr     r1, r0, #0x12	//ARM_MODE_IRQ
        msr     CPSR, r1
        mov     sp, #0x800	//set stack pointer for irq mode
        orr     r1, r0, #0x13	//ARM_MODE_ABORT
        msr     CPSR, r1
        mov     sp, #0xC00	//set stack pointer for abort mode
        orr     r1, r0, #0x17	//ARM_MODE_supervisor
        msr     CPSR, r1
        mov     sp, #0x1000	//set stack pointer for supervisor mode
        orr     r1, r0, #0x1B	//ARM_MODE_UNDEFINED
        msr     CPSR, r1
        mov     sp, #0x1400	//set stack pointer for undefined mode
        orr     r1, r0, #0x1F	//ARM_MODE_SYS
        msr     CPSR, r1
        mov     sp, #0x4000	//set stack pointer for system/user mode
        "
        :
        :
        :
        :
    )}
    //Code anschalten der interrupts (IRQ + FIQ), von dem Register CPSR müssen jeweils bit 7 und 6 auf 0 gesetzt werden, damit die interrupts aufgerufen werden.
    //Zusätzlich müssen die Interrupts noch im advanced interrupt controller angeschaltet werden.
    unsafe{
        asm!("
            push {r0}
            mrs  r0, CPSR
            bic  r0, r0, #0b11000000	//enable irq, fiq
            msr  CPSR, r0
            pop {r0}"
            :
            :
            :
            :
        )
    }
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
