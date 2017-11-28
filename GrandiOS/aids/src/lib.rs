//this is our STD
//Advanced Interface for Dispatching Syscalls
#![no_std]
#![feature(asm)]

extern crate swi;

#[macro_extern]
macro_rules! init {
    () => (
        extern crate alloc;
        mod allocator;
        use allocator;
        #[global_allocator]
        static GLOBAL: allocator::Allocator = allocator::Allocator::new();
    );
}
#[macro_extern]
macro_rules! read {
    () => (
        let input      = swi::read::Input{};
        let mut output = swi::read::Output{c: 0};
        swi::read::call(& input, &mut output);
        output.c
    );
}
#[macro_extern]
macro_rules! switch {
    let input      = swi::switch::Input{};
    let mut output = swi::switch::Output{};
    swi::switch::call(& input, &mut output);
}
