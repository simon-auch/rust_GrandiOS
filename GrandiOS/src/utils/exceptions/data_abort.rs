use driver::serial::*;
use driver::interrupts::InterruptController;
use core::ptr::read_volatile;
use utils::registers;
use utils::vt;

pub fn init(ic : &mut InterruptController){
    //set the handler for the data abort interrupt
    ic.set_handler_data_abort(handler);
    println!("Exception handler data abort: 0x{:x}", handler as u32);
}

#[naked]
extern fn handler(){
    //the address of the data_abort instruction is r14-0x8, therefore we want to jump to r14-0x4 to leave the instruction behind.
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
        subs   pc, r14, 0x4"
        :
        :
        :
        :
    )}
}
fn handler_helper(lr: u32){
    let mut lr = lr - 0x8;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} data_abort at: 0x{:x}, instruction: 0x{:x}\n", &vt::CF_RED, &vt::CF_STANDARD, lr, instruction).unwrap();
}

pub fn provoke(){
    let _ : u32 = unsafe { read_volatile(0x400000 as *mut u32) };
    /*
    //Das sollte eigentlich auch funktionieren...
    unsafe{asm!("
        .word 0x0 //some nops to find it faster in the binary
        .word 0x0
        .word 0x0
        .word 0x0
        mov r0, 0x4000 << 8
        str r0, [r0]"
        :
        :
        :"r0"
        :"volatile"
    )}
    */
}
