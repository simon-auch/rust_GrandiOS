#![no_std]
#![feature(asm)]
#![feature(alloc,allocator_api)]

extern crate alloc;

use alloc::string::String;
pub struct TCBStatistics {
    pub id: u32,
    pub name: String,
    pub cpu_time: u32,
    pub priority: u32,
    pub memory_allocated: u32,
    pub memory_used: u32,
}



//macros that give the swi number of the corresponding swi
#[macro_export]
macro_rules! SWITCH {() => {0}; ( name ) => {switch};}
#[macro_export]
macro_rules! READ   {() => {1}; ( name ) => {read};}
#[macro_export]
macro_rules! WRITE  {() => {2}; ( name ) => {write};}
#[macro_export]
macro_rules! ALLOC  {() => {3}; ( name ) => {useralloc};}
 #[macro_export]
macro_rules! DEALLOC  {() => {4}; ( name ) => {userdealloc};}
#[macro_export]
macro_rules! GET_LED {() => {5}; ( name ) => {get_led};}
#[macro_export]
macro_rules! SET_LED {() => {6}; ( name ) => {set_led};}
#[macro_export]
macro_rules! SLEEP {() => {7}; ( name ) => {sleep};}
#[macro_export]
macro_rules! EXIT {() => {8}; ( name ) => {exit};}
#[macro_export]
macro_rules! SELECT {() => {9}; ( name ) => {select};}
#[macro_export]
macro_rules! IPC_WAIT {() => {10}; ( name ) => {ipc_wait};}
#[macro_export]
macro_rules! IPC_READ {() => {11}; ( name ) => {ipc_read};}
#[macro_export]
macro_rules! IPC_WRITE {() => {12}; ( name ) => {ipc_write};}
#[macro_export]
macro_rules! TCBS_STATISTICS {() => {13}; ( name ) => {tcbs_statistics};}

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

build_swi!(switch ; SWITCH   ; ; );
build_swi!(read   ; READ     ; ; c:u8);
build_swi!(write  ; WRITE    ; c:u8; );
build_swi!(useralloc  ; ALLOC    ; l:Layout; r:Option<Result<*mut u8, AllocErr>>; alloc::heap::Layout, alloc::heap::AllocErr);
build_swi!(userdealloc; DEALLOC  ; p:*mut u8, l:Layout; ; alloc::heap::Layout);
build_swi!(get_led; GET_LED  ; l:u8; s:bool);
build_swi!(set_led; SET_LED  ; l:u8, s:bool; );
build_swi!(sleep  ; SLEEP    ; t:u32; );
build_swi!(exit   ; EXIT     ; ; ;);
build_swi!(select ; SELECT   ; c:Vec<usize>, i:Vec<Vec<u8>>; c:usize; alloc::vec::Vec);
build_swi!(ipc_wait; IPC_WAIT ; c:usize; );
build_swi!(ipc_read; IPC_READ ; c:usize; p:Vec<u8>; alloc::vec::Vec);
build_swi!(ipc_write; IPC_WRITE; c:usize, i:Vec<u8>; ; alloc::vec::Vec);
build_swi!(tcbs_statistics; TCBS_STATISTICS; ; c:Vec<TCBStatistics>; alloc::vec::Vec,TCBStatistics);
