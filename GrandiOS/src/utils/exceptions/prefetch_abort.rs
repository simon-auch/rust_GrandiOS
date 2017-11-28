use driver::serial::*;
use driver::interrupts::InterruptController;
use core::ptr::read_volatile;
use utils::registers;
use utils::vt;

pub fn init(ic : &mut InterruptController){
    //set the handler for the prefetch abort interrupt
    ic.set_handler_prefetch_abort(handler);
    println!("Exception handler prefetch abort: 0x{:x}", handler as u32);
}

#[naked]
extern fn handler(){
    //TODO: keine ahnung ob das so richtig ist. sollte zumindest bis zum print kommen, kehrt aber nicht automatisch zurück  
    let mut debug_unit = unsafe { DebugUnit::new(0xFFFFF200) };
    write!(debug_unit, "{}Exception{} prefetch_abort. We dont handle this yet, going into a loop.", &vt::CF_RED, &vt::CF_WHITE).unwrap();
    loop{}
}

pub fn provoke(){
    println!("TODO: implement me!");//Geht ohne speicherschutz noch nicht
}
