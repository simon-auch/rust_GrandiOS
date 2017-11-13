#![no_std]
#![feature(lang_items)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(const_fn)]
#![feature(const_unsafe_cell_new)]
//disable some warnings
//#![allow(unused_variables)]
//#![allow(unused_imports)]
//#![allow(unused_unsafe)]
//#![allow(unused_mut)]
//#![allow(dead_code)]
//alloc needs lots of features
#![feature(alloc, global_allocator, allocator_api, heap_api)]
#![feature(compiler_builtins_lib)]
//Include other parts of the kernal

#[macro_use]
mod driver{
	#[macro_use]
	pub mod serial;
	pub mod led;
    pub mod logo;

	pub use serial::*;
	pub use led::*;
}
mod utils{
	pub mod spinlock;
	pub mod allocator;
    pub mod thread;
}
use driver::*;

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
    logo::draw();
    loop {
        let input = String::from_utf8(echo_readln!("> ")).expect("Found invalid UTF-8");
        let mut arguments: Vec<&str> = input.split(' ').collect();
        let command = arguments.remove(0);
        match command {
            "logo" => logo::draw(),
            "cat" => if arguments.len() == 0 {
                println!("{}", read!() as char);
            } else {
                println!("{}", read!());
            },
            "test" => if arguments.len() == 0 { println!("Test what?"); } else {
                match arguments[0].as_ref() {
                    "size" => {
                        let (w, h) = logo::resize();
                        println!("{}x{}",w,h);
                    },
                    "alloc" => {
                        {
                            let a = Box::new("Hallo");
                            let b = Box::new("Welt!");
                            println!("{} at {:p}", a, a);
                            println!("{} at {:p}", b, b);
                        }
                        let a = Box::new("Test");
                        println!("{} at {:p}", a, a);
                    },
                    "lock" => {
                        let lock = utils::spinlock::Spinlock::new(0u32);
                        {
                            //lock is hold until data goes out of scope
                            let mut data = lock.lock();
                            *data += 1;

                            led_yellow.on();
                            let mut data2 = lock.try_lock();
                            match data2{
                                Some(guard) => {
                                    //we got the lock, but it should have been locked already..............
                                    led_red.on();},
                                None => {
                                    led_green.on();},
                            }
                        }
                    },
                    "tcb" => {
                        {// TCBs
                            let mut t1 = utils::thread::TCB::new(1,"Erster TCB");
                            let mut t2 = utils::thread::TCB::new(2,"Zweiter TCB");
                            t1.get_state();
                            
                            println!("[{1}] -- {0:?}: {2}", t1.update_state(), t1.id, t1.name);
                            println!("[{1}] -- {0:?}: {2}", t2.update_state(), t2.id, t2.name);
                            t2.save_registers();
                            t1.load_registers();
                        }
                    },
                    _ => println!("I don't know that.")
                };
            },
            _ => println!("Unknown command: {}", command)
        };
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
