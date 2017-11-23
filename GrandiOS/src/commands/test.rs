use driver::serial::*;
use utils::shell::*;
use utils::parser::Argument;
use core::result::Result;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{String,ToString};
use commands::logo;
use driver::led::*;
use driver::interrupts::*;
use utils::spinlock::*;
use utils::thread::*;
use utils::registers;
use utils::vt;
use core::ptr::{write_volatile, read_volatile};

pub fn exec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    eval_args(&mut args, 0);
    if args.len() == 0 { return Err("Test what?".to_string()); }
    if !args[0].is_method() { return Err("String expected".to_string()); }
    let tests = vec![
        ("size", test_size as fn()),
        ("alloc", test_alloc as fn()),
        ("lock", test_lock as fn()),
        ("tcb", test_tcb as fn()),
        ("vt_color", test_vt_color as fn()),
        ("interrupts_aic", test_interrupts_aic as fn()),
        ("interrupts_undefined_instruction", test_interrupts_undefined_instruction as fn()),
        ("interrupts_software_interrupt", test_interrupts_software_interrupt as fn()),
        ("interrupts_prefetch_abort", test_interrupts_prefetch_abort as fn()),
        ("interrupts_data_abort", test_interrupts_data_abort as fn()),
        ("undefined_instruction", test_undefined_instruction as fn()),
        ("software_interrupt", test_software_interrupt as fn()),
        ("prefetch_abort", test_prefetch_abort as fn()),
        ("data_abort", test_data_abort as fn())];
    let test_wanted = args[0].get_method_name().unwrap();
    if test_wanted == "help" {
        println!("Available tests:");
        for &(test_str, _) in &tests{
            print!("{} ", test_str);
        }
        println!("");
        return Ok(vec![]);
    }
    for (test_str, test_f) in tests{
        if test_str == test_wanted {
            test_f();
            return Ok(vec![]);
        }
    }
    Err("I don't know that.".to_string())
}


fn test_size(){
    let (w, h) = vt::get_size();
    println!("{}x{}",w,h);
}

pub fn test_alloc(){
    {
        let a = Box::new("Hallo");
        let b = Box::new("Welt!");
        println!("{} at {:p}", a, a);
        println!("{} at {:p}", b, b);
    }
    let a = Box::new("Test");
    println!("{} at {:p}", a, a);
}

fn test_lock(){
    let mut led_yellow = unsafe { PIO::new(PIO_LED_YELLOW) };
    let mut led_red    = unsafe { PIO::new(PIO_LED_RED)    };
    let mut led_green  = unsafe { PIO::new(PIO_LED_GREEN)  };
    let lock = Spinlock::new(0u32);
    {
        //lock is hold until data goes out of scope
        let mut data = lock.lock();
        *data += 1;

        led_yellow.on();
        let mut data2 = lock.try_lock();
        match data2{
            Some(guard) => {
                //we got the lock, but it should have been locked already..............
                led_red.on();},
            None => {
                led_green.on();},
        }
    }
}

fn test_tcb(){
    //TCB again
    // Take a fn-pointer, make it a rawpointer
    let idle_thread_function_ptr: *mut _ = idle_thread as *mut _;
    // Shove it into the TCB
    let mut tcb = TCB::new("Test TCB",idle_thread_function_ptr);
    let mut tcb2 = TCB::new("Test TCB2",idle_thread2 as *mut _);
    println!("[{1}] -- {0:?}: {2}", tcb.update_state(), tcb.id, tcb.name);
    //println!("pc...? {:p}",tcb.program_counter);
    //tcb.save_registers();
    //println!("pc...? {:p}",tcb.program_counter);
    tcb.load_registers();
    //tcb.save_registers();

    tcb2.load_registers();
    //tcb2.save_registers();

    tcb.load_registers();
    //tcb.save_registers();
    //tcb2.load_registers();
    //tcb.load_registers();
    //println!("pc...? {:p}",tcb.program_counter);
}

fn test_vt_color(){
    println!("{}Red on Black {}White on Black {}{}Red on Green {}{}White on Black{}{}{} Standard", &vt::CF_RED, &vt::CF_WHITE, &vt::CF_RED, &vt::CB_GREEN, &vt::CF_WHITE, &vt::CB_BLACK, &vt::ATT_RESET, &vt::CF_STANDARD, &vt::CB_STANDARD);
    println!("\x1B[38;2;255;0;0m 24-Bit Color? {}", &vt::CF_STANDARD);
    println!("8-Bit Color Table:");
    for i in 0..16{
        for j in 0..16{
            print!("{}{:03} ", &vt::Color{ct: vt::ColorType::Background, cc: vt::ColorCode::Bit8(i*16+j)}, (i*16+j))
        }
        println!("{}", &vt::CB_STANDARD);
    }
}

fn test_interrupts_aic(){
    //enable interrupts
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.set_handler(1, handler_irq); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.set_priority(1, 0);
    ic.set_sourcetype(1, 3);//positive edge triggered
    ic.enable();
    DEBUG_UNIT.interrupt_set_rxrdy(true);
    loop{}
}

#[naked]
extern fn handler_irq(){
    //IRQ_ENTRY from AT91_interrupts.pdf
    //Note: ldmfd/stmfd sp! is equivalent to pop/push and according to the docs the better way
    //TODO die stack pointer für den irq modus und den system/user modus muss zuerst noch gesetzt werden (beim system start)
    unsafe{asm!("
        sub     r14, r14, #4
        push    {r14}
        mrs     r14, SPSR
        push    {r0, r14}
        mrs     r14, CPSR
        bic     r14, r14, #0x80 //I_BIT
        orr     r14, r14, #0x1F //ARM_MODE_SYS
        msr     CPSR, r14
        push    {r1-r3, r4-r11, r12, r14}"
        :
        :
        :
        :
    )}
    {//this block is here to make sure destructors are called if needed.
        //TODO: find out what threw the interrupt.
        let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
        debug_unit.read(true); //read in echo mode
        //IRQ_EXIT from AT91_interrupts.pdf
    }
    unsafe{asm!("
        pop     {r1-r3, r4-r11, r12, r14}
        mrs     r0, CPSR
        bic     r0, r0, #0x1F //clear mode bits
        orr     r0, r0, #0x92 //I_BIT | ARM_MODE_IRQ
        msr     CPSR, r0
        ldr     r0, = 0xFFFFF000 //AIC_BASE
        str     r0, [r0, #0x0130] //AIC_EOICR
        pop     {r0, r14}
        msr     SPSR, r14
        ldmfd   sp!, {pc}^"
        :
        :
        :
        :
    )}
}

fn test_interrupts_undefined_instruction(){
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the undefined instruction interrupt
    ic.set_handler_undefined_instruction(handler_undefined_instruction);
    println!("Exception handler undefined instruction: 0x{:x}", handler_undefined_instruction as u32);
}

#[naked]
extern fn handler_undefined_instruction(){
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
        handler_undefined_instruction_helper(lreg);
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
fn handler_undefined_instruction_helper(lr: u32){
    let mut lr = lr - 0x4;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} undefined_instruction at: 0x{:x}, instruction: 0x{:x}\n", &vt::CF_RED, &vt::CF_STANDARD, lr, instruction).unwrap();
}

fn test_interrupts_software_interrupt(){
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the software interrupt
    ic.set_handler_software_interrupt(handler_software_interrupt);
    println!("Exception handler software interrupt: 0x{:x}", handler_software_interrupt as u32);
}
#[naked]
extern fn handler_software_interrupt(){
    //the address of the swi instruction is r14-0x4, therefore we want to jump to r14 to leave the swi instruction behind.
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
        //Zurzeit können wir nur SWI erstellen, die nur den direkten wert als parameter haben.
        let lreg = registers::get_lr();
        handler_software_interrupt_helper(lreg);
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
fn handler_software_interrupt_helper(lr: u32){
    let mut lr = lr - 0x4;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let immed = instruction & 0xFFFFFF;
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} software_interrupt at: 0x{:x}, instruction: 0x{:x}, swi value: 0x{:x}\n", &vt::CF_YELLOW, &vt::CF_STANDARD, lr, instruction, immed).unwrap();
}

fn test_interrupts_prefetch_abort(){
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the prefetch abort interrupt
    ic.set_handler_prefetch_abort(handler_prefetch_abort);
    println!("Exception handler prefetch abort: 0x{:x}", handler_prefetch_abort as u32);
}
#[naked]
extern fn handler_prefetch_abort(){
    //TODO: keine ahnung ob das so richtig ist. sollte zumindest bis zum print kommen, kehrt aber nicht automatisch zurück  
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} prefetch_abort. We dont handle this yet, going into a loop.", &vt::CF_RED, &vt::CF_WHITE).unwrap();
    loop{}
}
fn test_interrupts_data_abort(){
    //get interrupt controller, initialises some instruction inside the vector table too
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    //set the handler for the data abort interrupt
    ic.set_handler_data_abort(handler_data_abort);
    println!("Exception handler data abort: 0x{:x}", handler_data_abort as u32);
}
#[naked]
extern fn handler_data_abort(){
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
        handler_data_abort_helper(lreg);
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
fn handler_data_abort_helper(lr: u32){
    let mut lr = lr - 0x8;
    let instruction = unsafe { read_volatile(lr as *mut u32) };
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} data_abort at: 0x{:x}, instruction: 0x{:x}\n", &vt::CF_RED, &vt::CF_STANDARD, lr, instruction).unwrap();
}

fn test_undefined_instruction(){
    unsafe{asm!("
        .word 0xFFFFFFFF"
        :
        :
        :
        :
    )}
    println!("Handler undefined_instrucion returned")
}
fn test_software_interrupt(){
    unsafe{asm!("
        swi 0x80"
        :
        :
        :
        :
    )}
    println!("Handler software_interrupt returned");
}
fn test_prefetch_abort(){
    println!("TODO: implement me!");//Geht ohne speicherschutz noch nicht
}
fn test_data_abort(){
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
    println!("Handler data_abort returned");
}

