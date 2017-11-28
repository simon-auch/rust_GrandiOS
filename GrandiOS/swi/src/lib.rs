#![no_std]
#![feature(lang_items)]
#![feature(asm)]

//macros that give the swi number of the corresponding swi
#[macro_export]
macro_rules! SWITCH {() => {0};}
#[macro_export]
macro_rules! READ   {() => {1};}
#[macro_export]
macro_rules! WRITE  {() => {2};}

//creates the input and output structs with the given types and identifiers
macro_rules! IO {
    ($($in:ident : $ti:ty),*; $($out:ident : $to:ty),*) => (
        #[repr(C)]
        pub struct Input{
            $(pub $in: $ti),*
        }
        #[repr(C)]
        pub struct Output{
            $(pub $out: $to),*
        }
    );
}

//creates a call function given a Input and Output of the corresponding swi
macro_rules! CALL {
    ($num:tt ) => (
        pub fn call(input: & Input, output: &mut Output) {
            unsafe{asm!(concat!("swi ", $num!())
                : //outputs
                : "{r0}"(output), "{r1}"(input)//inputs
                :"memory" //clobbers
                :"volatile");}
        }
    );
}

//builds an swi call function and structs needed.
macro_rules! build_swi {
    ($name_mod:ident, $name_macro:ident; $($in:ident : $ti:ty),*; $($out:ident : $to:ty),*) => (
        pub mod $name_mod {
            IO!($($in : $ti),*; $($out : $to),*);
            CALL!($name_macro);
        }
    );
}

#[derive(Clone, Copy, Debug)]
pub enum SWI{
    Read{input: *mut read::Input, output: *mut read::Output},
}
build_swi!(switch, SWITCH; ; );
build_swi!(read,   READ  ; ; c:u8);
build_swi!(write,  WRITE ; c:u8; );
