//this is our STD
//Advanced Interface for Dispatching Syscalls
#![no_std]
#![feature(asm)]
#![feature(const_fn)]
#![feature(alloc,allocator_api)]
#![allow(unused_macros)]
extern crate swi;
extern crate alloc;
pub mod allocator;
#[macro_export]
macro_rules! init {
    () => (
        #[macro_use]
        extern crate alloc;
        extern crate swi;
        extern crate corepack;
        use core::fmt;
        use core::fmt::Write;
        #[global_allocator]
        static GLOBAL: aids::allocator::Allocator = aids::allocator::Allocator::new();
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
    );
}
#[macro_export]
macro_rules! read {
    () => {{
        let input      = ::swi::read::Input{};
        let mut output = ::swi::read::Output{c: 0};
        ::swi::read::call(& input, &mut output);
        output.c
    }};
    ( $t:expr ) => {
        select!(SLEEP!(), $t; READ!(), );
    };
}
#[macro_export]
macro_rules! print {
	( $x:expr ) => {
        unsafe {
		    write!(::PRINTER, $x).unwrap();
        }
	};
	( $($x:tt)* ) => {
        unsafe {
		    write!(::PRINTER, $($x)*).unwrap();
        }
	};
}
#[macro_export]
macro_rules! println {
	( $x:expr ) => {
        print!($x);
        print!("\n");
	};
	( $($y:tt )* ) => {
		print!($($y)*);
		print!("\n");
	};
}
#[macro_export]
macro_rules! switch {
    () => {{
        let input      = ::swi::switch::Input{};
        let mut output = ::swi::switch::Output{};
        ::swi::switch::call(& input, &mut output);
    }};
}
#[macro_export]
macro_rules! get_led {
    ( $l:expr ) => {{
        let input      = ::swi::get_led::Input{l:$l};
        let mut output = ::swi::get_led::Output{s:false};
        ::swi::get_led::call(& input, &mut output);
        output.s
    }};
}
#[macro_export]
macro_rules! set_led {
    ( $l:expr, $s:expr ) => {{
        let input      = ::swi::set_led::Input{l:$l, s:$s};
        let mut output = ::swi::set_led::Output{};
        ::swi::set_led::call(& input, &mut output);
    }};
}
#[macro_export]
macro_rules! sleep {
    ( $t:expr ) => ({
        let input      = ::swi::sleep::Input{t:$t};
        let mut output = ::swi::sleep::Output{};
        ::swi::sleep::call(& input, &mut output);
    });
}
#[macro_export]
macro_rules! exit {
    () => ({
        let input      = ::swi::exit::Input{};
        let mut output = ::swi::exit::Output{};
        ::swi::exit::call(& input, &mut output);
    });
}

macro_rules! generate_input {
    ( $channel:expr, $input:expr ) => {{
        match $channe {
            READ!() => { corepack::to_bytes(::swi::read::Input{}).unwrap() },
            SLEEP!() => { corepack::to_bytes(::swi::sleep::Input{t:$input}).unwrap() },
            _ => { corepack::to_bytes(::swi::ipc_read::Input{c:$channel, i:corepack::to_bytes($input).unwrap()}).unwrap() }
        }
    }};
}
#[macro_export]
macro_rules! select {
    ( $($c:expr, $i:expr);* ) => ({
        let c = vec![];
        let i = vec![];
        $(
            c.push($c);
            i.push(generate_input!($c, $i));
        )*
        let input      = ::swi::select::Input{c:c, i:i};
        let mut output = ::swi::select::Output{c:0};
        ::swi::select::call(& input, &mut output);
        output.c
    });
}
#[macro_export]
macro_rules! ipc_read {
    ( $channel:expr ) => {{
         let winput      = ::swi::ipc_wait::Input{c:$channel};
         let mut woutput = ::swi::ipc_wait::Output{};
         ::swi::ipc_wait::call(& input, &mut output);
         let mut result = Vec::with_capacity(o.s);
         let rinput      = ::swi::ipc_read::Input{c:$channel};
         let mut routput = ::swi::ipc_read::Output{p:result};
         ::swi::ipc_read::call(& rinput, &mut routput);
         output.p
    }};
}
#[macro_export]
macro_rules! tcbs_statistics {
    () => {{
        let input = ::swi::tcbs_statistics::Input{};
        let mut output = ::swi::tcbs_statistics::Output{c:vec!()};
        ::swi::tcbs_statistics::call(&input,&mut output);
        output.c
    }};
}
