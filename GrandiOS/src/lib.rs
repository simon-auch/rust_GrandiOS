#![no_std]
#![feature(lang_items)]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]
//Include other parts of the kernal
mod utils{
	pub mod spinlock;
}
mod driver{
	pub mod serial;
	pub mod led;
}

//#[no_mangle]
//keep the function name so we can call it from assembler
//pub extern
//make the function use the standard C calling convention
#[no_mangle]
#[naked]
pub extern fn _start() {
	let mut led_yellow = unsafe { driver::led::PIO::new(driver::led::PIO_LED_YELLOW) };
	let mut led_red    = unsafe { driver::led::PIO::new(driver::led::PIO_LED_RED)    };
	let mut led_green  = unsafe { driver::led::PIO::new(driver::led::PIO_LED_GREEN)  };
	led_yellow.on();
	led_red.on();
	led_green.on();

	driver::serial::print();
	let lock = utils::spinlock::Spinlock::new(0);
	{
		//lock is hold until data goes out of scope
		let mut data = lock.lock();
		* data += 1;
	}
	loop {}
}


// These functions and traits are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
fn panic_fmt() -> ! { loop {} }
