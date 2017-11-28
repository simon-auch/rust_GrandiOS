//Syscalls interface
//How should a syscall look like (example read_char):
// 1. reserve space for the return value of the syscall
// 2. create a pointer to the reserved space for the return value
// 3. reserve space for the parameters of the syscall
// 4. create a pointer to the reserved space for the parameters
// 5. move the pointer for the return values into r0
// 6. move the pointer for the parameters into r1
// 7. call the swi instruction with the correct number.

//Important note from the docu for the push, pop operations:
//"Registers are stored on the stack in numerical order, with the lowest numbered register at the lowest address."

use core::ptr::read_volatile;
use driver::interrupts::*;
use driver::serial::*;
use utils::irq;
use utils::vt;
use utils::registers;
use utils::scheduler;
use utils::thread::{TCB, State};

pub fn init() {
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the software interrupt
    ic.set_handler_software_interrupt(handler_software_interrupt);
    //irq.enable();
}

//This represantates the memory layout that gets pushed onto the stack when the interrupt starts.
macro_rules! build_register_stack {
    ($($name:ident),*) => (
        #[derive(Debug, Clone, Default)]
        #[repr(C)]
        pub struct RegisterStack {
            $(pub $name: u32),*
        }
        impl RegisterStack {
            pub fn copy(&mut self, source: & Self){
                $(self.$name = source.$name);*
            }
        }
    );
}
build_register_stack!(sp, cpsr, r0, r1, r2, r3, r4, r5, r6, r7, r8, r9, r10, r11, r12, lr);

//macros that give the swi number of the corresponding swi
macro_rules! SWITCH {() => {0};}
macro_rules! READ   {() => {1};}
macro_rules! WRITE  {() => {2};}

pub mod swi {
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
    build_swi!(read, READ; ; c:u8);
    build_swi!(write, WRITE; c:u8;);
}

#[naked]
extern fn handler_software_interrupt(){
    //the address of the swi instruction is r14-0x4, therefore we want to jump to r14 to leave the swi instruction behind.
    let reg_sp: u32;
    unsafe{asm!("
        push   {r14}
        push   {r0-r12}  //save everything except the Stack pointer (useless since we are in the wrong mode) and r0 as we want to write our result to there
        mrs    r0, SPSR  //move the program status from the interrupted program into r0
        push   {r0}
        mrs    r2, CPSR  //switch to ARM_MODE_SYS to save the stack pointer
        mov    r1, r2
        orr    r1, #0x1F
        msr    CPSR, r1
        mov    r0, sp    //move the stack pointer from thread into r0
        msr    CPSR, r2  //get back to interrupt mode
        push   {r0}
        mov    r0, sp    //move the stackpointer to r0 to know where r0-r12,r14 is stored
        sub    sp, 0x40" //make a bit of space on the stack for rust, since rust creates code like: "str r0, [pc, #4]" it expects the sp to be decremented before once. The 0x40 is a random guess and provides space for a few variables.
        :"={r0}"(reg_sp)
        :::
    )}
    {//this block is here to make sure destructors are called if needed.
        handler_software_interrupt_helper(reg_sp);
    }
    unsafe{asm!("
        add    sp, 0x40
        pop    {r0}
        mrs    r2, CPSR  //switch to ARM_MODE_SYS to save the stack pointer
        mov    r1, r2
        orr    r1, #0x1F
        msr    CPSR, r1
        mov    sp, r0
        msr    CPSR, r2
        pop    {r0}
        msr    SPSR, r0
        pop    {r0-r12}
        pop    {r14}
        movs   pc, r14"
        ::::
    )}
}

fn handler_software_interrupt_helper(reg_sp: u32){
    let regs = unsafe{ &mut(*(reg_sp as *mut RegisterStack)) };
    let instruction = unsafe { read_volatile((regs.lr - 0x4) as *mut u32) };
    let immed = instruction & 0xFFFFFF;
    let mut sched = unsafe {scheduler::get_scheduler()};

    match immed {
        SWITCH!() => {
            let input  = regs.r1 as *mut swi::switch::Input;
            let output = regs.r0 as *mut swi::switch::Output;
            sched.switch(regs);
        },
        READ!() => {
            let input  = regs.r1 as *mut swi::read::Input;
            let output = regs.r0 as *mut swi::read::Output;
            sched.get_current_tcb().state = State::Waiting(swi::SWI::Read{input: input, output: output});
            sched.switch(regs);
        },
        _ => {
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}, registers:{:?}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, regs.lr - 0x4, instruction, immed, regs).unwrap();
            write!(debug_unit, "{}Unknown SWI{}", &vt::CF_RED, &vt::CF_STANDARD).unwrap();
        }
    }
}
