use driver::interrupts::*;
use utils::scheduler;
use utils::exceptions::software_interrupt::*;

//imports for possible interrupt sources
use driver::serial::*;

pub fn init(ic : &mut InterruptController, debug_unit : & DebugUnitWrapper){
    ic.set_handler(1, handler_line_1); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.enable();
    debug_unit.interrupt_set_rxrdy(true);
}

#[naked]
extern fn handler_line_1(){
    //the address stored in r14 must be adjusted by -4 to correctly resume the interrupted thread.
    let reg_sp: u32;
    unsafe{asm!("
        sub    r14, #4
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
        sub    sp, 0x40" //make a bit of space on the stack for rust, since rust creates code like: "str r0, [pc, #4]" it expects the sp to be decremented before once. The 0x40 is a random guess and provides space for a few variabl$
        :"={r0}"(reg_sp)
        :::
    )}
    {//this block is here to make sure destructors are called if needed.
        handler_helper_line_1(reg_sp);
        let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
        ic.end_of_interrupt();
    }
    unsafe{asm!("
        add    sp, 0x40
        pop    {r0}
        mrs    r2, CPSR  //switch to ARM_MODE_SYS to restore the stack pointer
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
fn handler_helper_line_1(reg_sp: u32){
    let regs = unsafe{ &mut(*(reg_sp as *mut RegisterStack)) };
    let mut sched = unsafe {scheduler::get_scheduler()};
    //find out who threw the interrupt.
    let mut debug_unit = unsafe { DebugUnit::new(DUMM_BASE_ADRESS) };
    match debug_unit.read_nonblocking(false) {
        None => {},
        Some(c) => {
            sched.push_queue_waiting_read_input(c);
        },
    }
    //call switch just in case a new process was made available
    sched.switch(regs, scheduler::State::Ready);
}
