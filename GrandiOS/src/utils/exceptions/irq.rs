use driver::interrupts::*;
use utils::scheduler;
use utils::exceptions::common_code::RegisterStack;

//imports for possible interrupt sources
use driver::serial::*;

pub fn init(ic : &mut InterruptController, debug_unit : & DebugUnitWrapper){
    ic.set_handler(1, handler_line_1); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.enable();
    debug_unit.interrupt_set_rxrdy(true);
}

#[naked]
extern fn handler_line_1(){
    unsafe{
        trampolin!(4, handler_helper_line_1);
    }
}

#[inline(never)]
extern fn handler_helper_line_1(reg_sp: u32){
    let regs = unsafe{ &mut(*(reg_sp as *mut RegisterStack)) };
    let mut sched = unsafe {scheduler::get_scheduler()};
    //find out who threw the interrupt.
    //println!("handler_helper_line_1");
    let mut debug_unit = unsafe { DebugUnit::new(DUMM_BASE_ADRESS) };
    match debug_unit.read_nonblocking(false) {
        None => {},
        Some(c) => {
            sched.push_queue_waiting_read_input(c);
        },
    }
    //call switch just in case a new process was made available
    //println!("call switch from irq helper");
    sched.switch(regs, scheduler::State::Ready);
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.end_of_interrupt();
}
