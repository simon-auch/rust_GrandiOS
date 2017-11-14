use alloc::boxed::Box;
use driver::serial::*;

#[derive(Copy,Clone,Debug)]
pub enum State{
    // TODO: Welche wollen wir alle haben?
    Running,
    Ready,
    Waiting,
    Terminated,
}
pub struct TCB<'a> {
    //TODO
    pub id: u32,
    pub name: &'a str,
    // scheduling information
    state: State,
    cpu_time: u32,
    // ...
    boxed_program_ptr: Box<*mut u32>,
    pub program_counter: *mut u32, //R13
    stack_ptr: u32, //R15
}

//TODO: sinnvollere ID Verwaltung / ID-Vergabe-Service (?)
static mut NEXT_ID: u32 = 0;

impl<'a> TCB<'a> {
    pub fn new(name: &'a str,boxed_program_ptr: Box<*mut u32>) -> Self {
        let id;
        unsafe{
            NEXT_ID+=1;
            id=NEXT_ID;
        }
        println!("Created TCB with pc=\t{:p}",*boxed_program_ptr);
        TCB {
            id: id,
            name: name,
            state: State::Waiting,
            program_counter: *boxed_program_ptr,
            stack_ptr: 0,
            cpu_time: 0,
            boxed_program_ptr: boxed_program_ptr,
        }
    }

    pub fn update_state(&mut self) -> State {
        println!("TODO: Implement me! @TCB.update_state()");
        self.state=State::Ready;
        self.state
    }

    pub fn load_registers(&self) {
        //TODO: SP = R13,
        println!("TODO: @TCB.load_registers() [SP, check whether running]");
        let pc:*mut u32=self.program_counter;
        //println!("loading stored pc=\t{:p}",pc);
        unsafe{
        //TODO: Wenn stack existiert auf richtige reigenfolge prüfen etc. (works for now)
        //asm!("LDMFD R13!, {R0-R12, R14}");
        asm!("MOV R15, $0"::"r"(pc):"R0-R15"); // R15=PC
        }
    }

    pub fn save_registers(&mut self) {
        //TODO: SP = R13,
        println!("TODO: @TCB.save_registers() [SP]");
        let pc:*mut u32;
        unsafe{
        //TODO: Wenn stack existiert auf richtige reigenfolge prüfen etc. (works for now)
        //asm!("STMFD R13!, {R0-R12, R14}");
        asm!("MOV $0, R15":"=r"(pc)::"R0-R15":); // R15=PC
        }
        self.program_counter = pc;
        //println!("saving pc=\t{:p}",self.program_counter);
    }
}

//IDLE Thread
pub fn idle_thread() {
    println!("Idling..");
    loop{
    }
}
