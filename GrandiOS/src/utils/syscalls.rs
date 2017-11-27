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

pub fn init() {
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the software interrupt
    ic.set_handler_software_interrupt(handler_software_interrupt);
    //irq.enable();
}

//This represantates the memory layout that gets pushed onto the stack when the interrupt starts.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct RegisterStack{
    pub sp:   u32,
    pub cpsr: u32,
    pub r0:   u32,
    pub r1:   u32,
    pub r2:   u32,
    pub r3:   u32,
    pub r4:   u32,
    pub r5:   u32,
    pub r6:   u32,
    pub r7:   u32,
    pub r8:   u32,
    pub r9:   u32,
    pub r10:  u32,
    pub r11:  u32,
    pub r12:  u32,
    pub lr:   u32,
}
impl RegisterStack{
    pub fn copy(&mut self, source: & Self){
        self.sp   = source.sp;
        self.cpsr = source.cpsr;
        self.r0   = source.r0;
        self.r1   = source.r1;
        self.r2   = source.r2;
        self.r3   = source.r3;
        self.r4   = source.r4;
        self.r5   = source.r5;
        self.r6   = source.r6;
        self.r7   = source.r7;
        self.r8   = source.r8;
        self.r9   = source.r9;
        self.r10  = source.r10;
        self.r11  = source.r11;
        self.r12  = source.r12;
        self.lr   = source.lr;
    }
}

pub mod swi{
    pub const SWITCH : u32 = 0;
    pub const READ   : u32 = 1;
    pub const WRITE  : u32 = 2;
    #[derive(Clone, Copy, Debug)]
    pub enum SWI{
        Read{input: *mut read::Input, output: *mut read::Output},
    }
    pub mod read{
        use utils::thread::{TCB, State};
        use utils::syscalls::swi;
        pub struct Input{
        }
        pub struct Output{
            pub c: u8,
        }
        pub fn call(input: Input, output: Output) {
            unsafe{asm!(concat!("swi ", stringify!(swi::READ))
                : //outputs
                : "{r0}"(&output), "{r1}"(&input)//inputs
                :"memory" //clobbers
                :"volatile");}
        }
        pub fn work(input: *mut Input, output: *mut Output, tcb: &mut TCB){
            tcb.state = State::Waiting(swi::SWI::Read{input: input, output: output});
        }
    }
    pub mod switch{
        use utils::thread::{TCB, State};
        use utils::syscalls::swi;
        pub struct Input{}
        pub struct Output{}
        pub fn call(input: Input, output: Output) {
            unsafe{asm!(concat!("swi ", stringify!(swi::SWITCH))
                : //outputs
                : "{r0}"(&output), "{r1}"(&input)//inputs
                :"memory" //clobbers
                :"volatile");}
        }
        pub fn work(input: *mut Input, output: *mut Output, tcb: &mut TCB){
        }
    }
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

    match immed {
        swi::SWITCH => {
            let input  = regs.r1 as *mut swi::switch::Input;
            let output = regs.r0 as *mut swi::switch::Output;
            let tcb    = scheduler::get_current_tcb();
            swi::switch::work(input, output, tcb);
            tcb.save_registers(&regs);
            let next   = scheduler::select();
            next.load_registers(regs);
        },
        swi::READ => {
            let input  = regs.r1 as *mut swi::read::Input;
            let output = regs.r0 as *mut swi::read::Output;
            let tcb    = scheduler::get_current_tcb();
            swi::read::work(input, output, tcb);
            tcb.save_registers(&regs);
            let next   = scheduler::select();
            next.load_registers(regs);
        },
        _ => {
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}, registers:{:?}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, regs.lr - 0x4, instruction, immed, regs).unwrap();
            write!(debug_unit, "{}Unknown SWI{}", &vt::CF_RED, &vt::CF_STANDARD).unwrap();
        }
    }
}
