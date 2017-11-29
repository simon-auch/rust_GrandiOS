#![no_std]
#![feature(asm)]
#![feature(alloc,allocator_api)]

extern crate alloc;

//macros that give the swi number of the corresponding swi
#[macro_export]
macro_rules! SWITCH {() => {0};}
#[macro_export]
macro_rules! READ   {() => {1};}
#[macro_export]
macro_rules! WRITE  {() => {2};}
#[macro_export]
macro_rules! ALLOC  {() => {3};}
#[macro_export]
macro_rules! DEALLOC  {() => {4};}

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
    ($name_mod:ident; $name_macro:ident; $($in:ident : $ti:ty),*; $($out:ident : $to:ty),*) => (
        build_swi!($name_mod; $name_macro; $($in:$ti),*; $($out:$to),*;);
    );
    ($name_mod:ident; $name_macro:ident; $($in:ident : $ti:ty),*; $($out:ident : $to:ty),*; $($use:path),*) => (
        pub mod $name_mod {
            $(use $use;)*
            IO!($($in : $ti),*; $($out : $to),*);
            CALL!($name_macro);
        }
    );
}

build_swi!(switch;      SWITCH ; ; );
build_swi!(read;        READ   ; ; c:u8);
build_swi!(write;       WRITE  ; c:u8; );
build_swi!(useralloc;   ALLOC  ; l:Layout; r:Option<Result<*mut u8, AllocErr>>; alloc::heap::Layout, alloc::heap::AllocErr);
build_swi!(userdealloc; DEALLOC; p:*mut u8, l:Layout; ; alloc::heap::Layout);
