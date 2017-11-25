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

#[naked]
extern fn handler_software_interrupt(){
    //the address of the swi instruction is r14-0x4, therefore we want to jump to r14 to leave the swi instruction behind.
    unsafe{asm!("
        push   {r14}
        push   {r1-r12}" //save everything except the Stack pointer (useless since we are in the wrong mode) and r0 as we want to write our result to there
        ::::
    )}
    {//this block is here to make sure destructors are called if needed.
        //load the memory location that threw the code
        //Zurzeit können wir nur SWI erstellen, die nur den direkten wert als parameter haben.
        let lreg = registers::get_lr();
        handler_software_interrupt_helper(lreg);
    }
    unsafe{asm!("
        pop    {r1-r12}
        pop    {r14}
        movs   pc, r14"
        ::::
    )}
}

fn handler_software_interrupt_helper(lr: u32){
    let mut lr = lr - 0x4;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let immed = instruction & 0xFFFFFF;
    match immed {
        1 => { //read
            let c = read!() as u32;
            unsafe{asm!("
                mov r0, $0
            " ::"r"(c)::"volatile");}
        },
        _ => {
            let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
            write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, lr, instruction, immed).unwrap();
        }
    }
}

pub fn read() -> u8 {
    let c: u32;
    unsafe{asm!("
        push {r0}
        swi 1
        mov $0, r0
    " :"=r"(c):::"volatile");}
    unsafe{asm!("
        pop {r0}
    " ::::"volatile");}
    c as u8
}
