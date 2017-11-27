use driver::serial::*;
use driver::interrupts::*;
use utils::syscalls;
use utils::ring::Ring;

pub fn enable(){
    //enable interrupts
    let mut ic = unsafe { InterruptController::new(IT_BASE_ADDRESS, AIC_BASE_ADDRESS) } ;
    ic.set_handler(1, handler_irq); //interrupt line 1 is SYS: Debug unit, clocks, etc
    ic.set_priority(1, 0);
    ic.set_sourcetype(1, 3);//positive edge triggered
    ic.enable();
    DEBUG_UNIT.interrupt_set_rxrdy(true);
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
