use driver::serial::*;
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

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() == 0 { return Err("Test what?".to_string()); }
    if !args[0].is_str() { return Err("String expected".to_string()); }
    match args[0].get_str().unwrap().as_str() {
        "size" => {test_size();},
        "alloc" => {test_alloc();},
        "lock" => {test_lock()},
        "tcb_1" => {test_tcb_1();},
        "tcb_2" => {test_tcb_2();},
        "interrupts" => {test_interrupts();},
        _ => return Err("I don't know that.".to_string())
    }
    Ok(vec![])
}

fn test_size(){
    let (w, h) = logo::resize();
    println!("{}x{}",w,h);
}

fn test_alloc(){
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

fn test_tcb_1(){
    println!("Baut nicht mehr, war im commit so drin, signatur von TCB::new hat sich wohl geändert");
    /*
    // TCBs
    let mut t1 = TCB::new(1,"Erster TCB");
    let mut t2 = TCB::new(2,"Zweiter TCB");
    t1.get_state();

    println!("[{1}] -- {0:?}: {2}", t1.update_state(), t1.id, t1.name);
    println!("[{1}] -- {0:?}: {2}", t2.update_state(), t2.id, t2.name);
    t2.save_registers();
    t1.load_registers();
    */
}

fn test_tcb_2(){
    //TCB again
    // Take a fn-pointer, make it a rawpointer
    let idle_thread_function_ptr: *mut _ = idle_thread as *mut _;
    // Box it
    let idle = Box::new(idle_thread_function_ptr);
    // Shove it into the TCB
    let mut tcb = TCB::new("Test TCB",idle);
    println!("[{1}] -- {0:?}: {2}", tcb.update_state(), tcb.id, tcb.name);
    //println!("pc...? {:p}",tcb.program_counter);
    //tcb.save_registers();
    //println!("pc...? {:p}",tcb.program_counter);
    tcb.load_registers();
    //println!("pc...? {:p}",tcb.program_counter);
}


fn test_interrupts(){
    //enable interrupts
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.set_handler(1, irq_handler); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.set_priority(1, 0);
    ic.set_sourcetype(1, 3);//positive edge triggered
    ic.enable();
    DEBUG_UNIT.interrupt_set_rxrdy(true);
    //Fasst der richtige code zum anschalten der interrupts (IRQ + FIQ), von dem Register CPSR müssen jeweils bit 7 und 6 auf 0 gesetzt werden, damit die interrupts aufgerufen werden.
    //TODO: das noch fixen, quelle für beispiel siehe irq_handler
    unsafe{
        asm!("
            push {r0}
            mrs  r0, CPSR
            bic  r0, r0, #0b11000000	//enable irq, fiq
            msr  CPSR, r0
            pop {r0}"
            :
            :
            :
            :
        )
    }
    loop{}
}

#[naked]
extern fn irq_handler(){
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
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    debug_unit.read(true); //read in echo mode
    //IRQ_EXIT from AT91_interrupts.pdf
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
        ldmfd  sp!, {pc}^ //In dem pdf steht hier {pc}^, das ist aber nicht erlaubt.."
        :
        :
        :
        :
    )}
}
