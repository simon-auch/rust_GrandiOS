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

use swi;
use core::ptr::read_volatile;
use driver::interrupts::*;
use driver::serial::*;
use driver::led::*;
use utils::vt;
use utils::scheduler;
use utils::thread::TCB;
use utils::allocator::*;
use alloc::allocator::Alloc;
use core::fmt;
use utils::exceptions::common_code::RegisterStack;

pub fn init(ic: &mut InterruptController) {
    //set the handler for the software interrupt
    ic.set_handler_software_interrupt(handler_software_interrupt);
    println!("Exception handler swi: 0x{:x}", handler_software_interrupt as u32);
}

#[naked]
extern fn handler_software_interrupt(){
    unsafe{
        trampolin!(0, handler_software_interrupt_helper);
    }
}

#[inline(never)]
extern fn handler_software_interrupt_helper(reg_sp: u32){
    let regs = unsafe{ &mut(*(reg_sp as *mut RegisterStack)) };
    let instruction = unsafe { read_volatile((regs.lr_irq - 0x4) as *mut u32) };
    let immed = instruction & 0xFFFFFF;
    let mut sched = unsafe {scheduler::get_scheduler()};
    match immed {
        SWITCH!() => {
            sched.switch(regs, scheduler::State::Ready);
        },
        READ!() => {
            sched.switch(regs, scheduler::State::WaitingRead);
        },
        WRITE!() => {
            let mut input : &mut swi::write::Input = unsafe{ &mut *(regs.r1 as *mut _) };
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}", input.c as char).unwrap();
        },
        ALLOC!() => {
            let mut input : &mut swi::useralloc::Input = unsafe{ &mut *(regs.r1 as *mut _) };
            let mut output: &mut swi::useralloc::Output = unsafe{ &mut *(regs.r0 as *mut _) };
            let layout = input.l.clone();
            let ptr = unsafe { Some((&mut &::GLOBAL).alloc(layout)) };
            output.r = ptr.clone();
            sched.alloc(ptr.unwrap().unwrap(), input.l.clone());
        },
        DEALLOC!() => {
            let mut input : &mut swi::userdealloc::Input = unsafe{ &mut *(regs.r1 as *mut _) };
            let layout = input.l.clone();
            unsafe {
                (&mut &::GLOBAL).dealloc(input.p, layout);
            }
        },
        GET_LED!() => {
            let mut input : &mut swi::get_led::Input = unsafe{ &mut *(regs.r1 as *mut _) };
            let mut output: &mut swi::get_led::Output = unsafe{ &mut *(regs.r0 as *mut _) };
            output.s = match input.l {
                0 => unsafe{ PIO::new(PIO_LED_RED).is_on() },
                1 => unsafe{ PIO::new(PIO_LED_YELLOW).is_on() },
                2 => unsafe{ PIO::new(PIO_LED_GREEN).is_on() },
                _ => false
            };
        },
        SET_LED!() => {
            let mut input : &mut swi::set_led::Input = unsafe{ &mut *(regs.r1 as *mut _) };
            match input.l {
                0 => unsafe{ PIO::new(PIO_LED_RED).set(input.s) },
                1 => unsafe{ PIO::new(PIO_LED_YELLOW).set(input.s) },
                2 => unsafe{ PIO::new(PIO_LED_GREEN).set(input.s) },
                _ => {}
            };
        }
        _ => {
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}, registers:{:?}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, regs.lr_irq - 0x4, instruction, immed, regs).unwrap();
            write!(debug_unit, "{}Unknown SWI{}", &vt::CF_RED, &vt::CF_STANDARD).unwrap();
        }
    }
}


//contains all the function to process a syscall, given the needed inputs. Will typically be called from the scheduler if the inputs are available and the corresponding syscall was called for that thread
//TODO: wenn we have a MMU we need to translate the addresses behind r0 and r1 before using them.
pub mod work {
    use utils::thread::TCB;
    use swi;

    pub fn read(tcb: &mut  TCB, c: u8){
        let mut output : &mut swi::read::Output = unsafe{ &mut *(tcb.register_stack.r0 as *mut _) };
        output.c = c;
    }
}
