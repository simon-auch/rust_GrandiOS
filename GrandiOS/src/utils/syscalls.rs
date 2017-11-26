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
use utils::vt;
use utils::registers;

pub fn init() {
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the software interrupt
    ic.set_handler_software_interrupt(handler_software_interrupt);
}

//This represantates the memory layout that gets pushed onto the stack when the interrupt starts.
#[derive(Debug)]
#[repr(C)]
struct register_stack{
    r0:  u32,
    r1:  u32,
    r2:  u32,
    r3:  u32,
    r4:  u32,
    r5:  u32,
    r6:  u32,
    r7:  u32,
    r8:  u32,
    r9:  u32,
    r10: u32,
    r11: u32,
    r12: u32,
    lr:  u32,
}

pub mod swi{
    pub mod read{
        use driver::serial::DEBUG_UNIT;
        pub struct Input{
        }
        pub struct Output{
            pub c: u8,
        }
        pub fn call() -> u8 {
            let mut output = Output{c: 0};
            let mut input  = Input{};
            unsafe{asm!("
                swi 1"
                : //outputs
                : "{r0}"(&output), "{r1}"(&input)//inputs
                :"memory" //clobbers
                :"volatile");}
            output.c
        }
        pub fn work(input: &mut Input, output: &mut Output){
            output.c = read!();
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
        pop    {r0-r12}
        pop    {r14}
        movs   pc, r14"
        ::::
    )}
}

fn handler_software_interrupt_helper(reg_sp: u32){
    let regs = unsafe{ &mut(*(reg_sp as *mut register_stack)) };
    let instruction = unsafe { read_volatile((regs.lr - 0x4) as *mut u32) };
    let immed = instruction & 0xFFFFFF;

    match immed {
        1 => { //read
            let input = unsafe{ &mut(*(regs.r1 as *mut swi::read::Input))};
            let output = unsafe{ &mut(*(regs.r0 as *mut swi::read::Output))};
            swi::read::work(input, output);
        },
        _ => {
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}, registers:{:?}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, regs.lr - 0x4, instruction, immed, regs).unwrap();
            write!(debug_unit, "{}Unknown SWI{}", &vt::CF_RED, &vt::CF_STANDARD).unwrap();
        }
    }
}
