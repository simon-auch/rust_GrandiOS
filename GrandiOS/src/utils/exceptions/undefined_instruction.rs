use driver::serial::*;
use driver::interrupts::InterruptController;
use core::ptr::read_volatile;
use utils::registers;
use utils::vt;

pub fn init(ic: &mut InterruptController){
    //set the handler for the undefined instruction interrupt
    ic.set_handler_undefined_instruction(handler);
    println!("Exception handler undefined instruction: 0x{:x}", handler as u32);
}

#[naked]
extern fn handler(){
    //the address of the undefined instruction is r14-0x4, therefore we want to jump to r14 to leave the problematic instruction behind.
    unsafe{asm!("
        push   {r14}
        push   {r0-r12}" //save everything except the Stack pointer (useless since we are in the wrong mode)
        :
        :
        :
        :
    )}
    {//this block is here to make sure destructors are called if needed.
        //load the memory location that threw the code
        let lreg = registers::get_lr();
        handler_helper(lreg);
    }
    unsafe{asm!("
        pop    {r0-r12}
        pop    {r14}
        movs   pc, r14" 
        :
        :
        :
        :
    )}
}
fn handler_helper(lr: u32){
    let mut lr = lr - 0x4;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} undefined_instruction at: 0x{:x}, instruction: 0x{:x}\n", &vt::CF_RED, &vt::CF_STANDARD, lr, instruction).unwrap();
}

pub fn provoke(){
    unsafe{asm!("
        .word 0xFFFFFFFF"
    )}
}
