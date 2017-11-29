//this is our STD
//Advanced Interface for Dispatching Syscalls
#![no_std]
#![feature(asm)]
#![feature(const_fn)]
#![feature(alloc,allocator_api)]
extern crate swi;
extern crate alloc;
pub mod allocator;
#[macro_export]
macro_rules! init {
    () => (
        #[macro_use]
        extern crate alloc;
        extern crate swi;
        use core::fmt;
        use core::fmt::Write;
        //#[global_allocator]
        //static GLOBAL: aids::allocator::Allocator = aids::allocator::Allocator::new();
        pub struct Printer;
        pub static mut PRINTER: Printer = Printer{};
        impl fmt::Write for Printer{
            fn write_char(&mut self, c: char) -> fmt::Result {
                let input      = swi::write::Input{c: c as u8};
                let mut output = swi::write::Output{};
                swi::write::call(& input, &mut output);
                Ok(())
            }
            fn write_str(&mut self, s: &str) -> fmt::Result {
                for c in s.chars(){
                    self.write_char(c).unwrap();
                }
                Ok(())
            }
            fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
                fmt::write(self, args)
            }
        }
    );
}
#[macro_export]
macro_rules! read {
    () => ({
        let input      = ::swi::read::Input{};
        let mut output = ::swi::read::Output{c: 0};
        ::swi::read::call(& input, &mut output);
        output.c
    });
}
#[macro_export]
macro_rules! print {
	( $x:expr ) => {
        unsafe {
		    write!(::PRINTER, $x).unwrap();
        }
	};
	( $x:expr, $( $y:expr ),* ) => {
        unsafe {
		    write!(::PRINTER, $x, $($y),*).unwrap();
        }
	};
}
#[macro_export]
macro_rules! println {
	( $x:expr ) => {
        print!($x);
        print!("\n");
	};
	( $x:expr, $( $y:expr ),* ) => {
		print!($x, $($y),*);
		print!("\n");
	};
}
#[macro_export]
macro_rules! switch {
    () => {
        let input      = ::swi::switch::Input{};
        let mut output = ::swi::switch::Output{};
        ::swi::switch::call(& input, &mut output);
    };
}
