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
//#![allow(unused_imports)]
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
    pub mod exceptions{
        pub mod data_abort;
        pub mod undefined_instruction;
        pub mod prefetch_abort;
        pub mod software_interrupt;
    }
    pub mod thread;
    pub mod scheduler;
    pub mod registers;
    pub mod ring;
    pub mod irq;
    extern crate vt;
}
use driver::*;
use alloc::string::ToString;

#[global_allocator]
pub static GLOBAL: utils::allocator::Allocator = utils::allocator::Allocator::new(0x22000000, 0x23ffffff);
#[macro_use]
extern crate alloc;
extern crate compiler_builtins;
extern crate rlibc;
#[macro_use]
extern crate swi;

extern {
    fn _shell_start();
}

//#[no_mangle]
//keep the function name so we can call it from assembler
//pub extern
//make the function use the standard C calling convention
#[no_mangle]
#[naked]
pub extern fn _start() {
    init_stacks();
    main();//call another function to make sure rust correctly does its stack stuff
    loop{};
}
fn main(){
    //make interupt table writable
    let mut mc = unsafe { MemoryController::new(MC_BASE_ADRESS) } ;
    mc.remap();

    //Initialise the DebugUnit
    DEBUG_UNIT.reset();
    DEBUG_UNIT.enable();
    
    println!("Waiting for keypress before continue");
    read!();

    //Initialisieren der Ausnahmen
    println!("Initialisiere Ausnahmen");
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    utils::exceptions::software_interrupt::init(&mut ic);
    utils::exceptions::data_abort::init(&mut ic);
    utils::exceptions::undefined_instruction::init(&mut ic);
    utils::exceptions::prefetch_abort::init(&mut ic);

    //Initialisieren der Interrupts
    println!("Initialisiere Interrupts");
    utils::irq::init(&mut ic, & DEBUG_UNIT);

    //Initialisieren des Schedulers
    println!("Initialisiere Scheduler");
    let mut tcb_current = utils::thread::TCB::new("Shell Thread".to_string(), _shell_start as *mut _, 0, 0); //function, memory, and cpsr will be set when calling the switch interrupt
    tcb_current.set_priority(10);
    //Initialise scheduler
    unsafe{ utils::scheduler::init(tcb_current) };

    //switch into user mode before starting the shell + enable interrupts, from this moment on the entire os stuff that needs privileges is done from syscalls (which might start privileged threads)
    unsafe{asm!("
        msr CPSR, r0"
        :
        :"{r0}"(utils::registers::CPSR_MODE_USER) //interrupts are enabled if the bits are 0
        :
        :"volatile"
    );}
    println!("Starte Shell");
    unsafe{_shell_start()};
}

#[inline(always)]
#[naked]
fn init_stacks(){
    //initialise the stack pointers for all modes.
    //each stack gets around 2kbyte, except the fiq which has a bit less (vector table+ jump addresses) and the system/user stack which has (16-2*6 = 4)kbyte
    unsafe{asm!("
        mov     r2, #0x200000
        mrs     r0, CPSR	//auslaesen vom status register
        bic     r0, r0, #0x1F	//set all mode bits to zero
        orr     r1, r0, #0x11	//ARM_MODE_FIQ
        msr     CPSR, r1
        add     r2, #0x800
        mov     sp, r2		//set stack pointer for fiq mode
        orr     r1, r0, #0x12	//ARM_MODE_IRQ
        msr     CPSR, r1
        add     r2, #0x800
        mov     sp, r2		//set stack pointer for irq mode
        orr     r1, r0, #0x13	//ARM_MODE_ABORT
        msr     CPSR, r1
        add     r2, #0x800
        mov     sp, r2		//set stack pointer for abort mode
        orr     r1, r0, #0x17	//ARM_MODE_supervisor
        msr     CPSR, r1
        add     r2, #0x800
        mov     sp, r2		//set stack pointer for supervisor mode
        orr     r1, r0, #0x1B	//ARM_MODE_UNDEFINED
        msr     CPSR, r1
        add     r2, #0x800
        mov     sp, r2		//set stack pointer for undefined mode
        orr     r1, r0, #0x1F	//ARM_MODE_SYS
        msr     CPSR, r1
        add     r2, #0x1800
        mov     sp, r2		//set stack pointer for system/user mode
        "
        :
        :
        :"memory"
        :"volatile"
    )}
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
