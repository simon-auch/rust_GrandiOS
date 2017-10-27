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
}


//#[no_mangle]
//keep the function name so we can call it from assembler
//pub extern
//make the function use the standard C calling convention
#[no_mangle]
#[naked]
pub extern fn _start() {
	driver::serial::print();
	foo();
	let lock = utils::spinlock::Spinlock::new(0);
	{
		//lock is hold until data goes out of scope
		let mut data = lock.lock();
		* data += 1;
	}
	loop {}
}

fn foo(){

}



// These functions and traits are used by the compiler, but not
// for a bare-bones hello world. These are normally
// provided by libstd.
#[lang = "eh_personality"]
extern fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
fn panic_fmt() -> ! { loop {} }
