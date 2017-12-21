use driver::interrupts::*;
use driver::system_timer::*;
use utils::scheduler;
use utils::exceptions::common_code::RegisterStack;
//Aufgabe 4
use utils::thread;
use swi;
use utils::registers;
use alloc::string::ToString;

//imports for possible interrupt sources
use driver::serial::*;

pub fn init(ic : &mut InterruptController){
    ic.set_handler(1, handler_line_1); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.enable();
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
            //sched.push_queue_waiting_read_input(c);
            //für aufgabe 4
            let mut tcb = thread::TCB::new("Thread".to_string(), p_ub_4 as *const (), 0x800, registers::CPSR_MODE_USER | registers::CPSR_IMPRECISE_ABORT);
            tcb.register_stack.r0=c as u32;
            tcb.set_priority(10);
            sched.add_thread(tcb);
        },
    }
    let mut st = unsafe{ get_system_timer() };
    let (pits, wdovf, rttinc, alms) = st.check_timers();//muss aufgerufen werden, da der interrupt ansonste ndirekt nochmal ausgeführt wird.
    if pits {
        //timer interrupt
        print!("!");
    }

    //call switch just in case a new process was made available
    //println!("call switch from irq helper");
    sched.switch(regs, scheduler::State::Ready);
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.end_of_interrupt();
}


extern fn p_ub_4(){
    let c: u32;
    unsafe{asm!(""
        :"={r0}"(c):::
    );}
    for i in 0..10 {
        let input      = swi::write::Input{c: c as u8};
        let mut output = swi::write::Output{};
        swi::write::call(& input, &mut output);
        for j in 0..100000 {}
/*
        let input      = ::swi::sleep::Input{100};
        let mut output = ::swi::sleep::Output{};
        let input_ref : u32 = ((&input) as *const _)as u32;
        let output_ref: u32 = ((&mut output) as *mut _) as u32;
        let input_arr : [u32; 2] = [1,input_ref];
        let output_arr: [u32; 2] = [1,output_ref];
        let select_input = ::swi::select::Input{swi_numbers: vec!(::swi::sleep::NUMBER), swi_inputs: vec!(input_ref)};
        let mut select_output = ::swi::select::Output{index: 0, swi_outputs: vec!(output_ref)};
        ::swi::select::call(& select_input, &mut select_output);
*/
    }
    let input = swi::exit::Input{};
    let mut output = swi::exit::Output{};
    swi::exit::call(& input, &mut output);
}

