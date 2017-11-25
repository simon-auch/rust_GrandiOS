use driver::serial::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use rlibc::memcpy;

#[derive(Copy,Clone,Debug)]
pub enum State{
    // TODO: Welche wollen wir alle haben?
    Created,
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
    stack_ptr: *mut u32, //R15
    registers: [u32;14],
}

//TODO: sinnvollere ID Verwaltung / ID-Vergabe-Service (?)
static mut NEXT_ID: u32 = 0;

impl<'a> TCB<'a> {
    pub fn new(name: &'a str,program_ptr: *mut u32) -> Self {
        let id;
        unsafe{
            NEXT_ID+=1;
            id=NEXT_ID;
        }

        println!("Created TCB with pc=\t{:p}",program_ptr);
        let mut regs: [u32;14] = Default::default();
        regs[13] = program_ptr as u32; //regs[13] ist das LR und der PC wird aus dem LR geladen
        TCB {
            id: id,
            name: name,
            state: State::Created,
            cpu_time: 0,
            registers: regs,
            stack_ptr: 0 as *mut _,
        }
    }

    pub fn update_state(&mut self) -> State {
        println!("TODO: Implement me! @TCB.update_state()");
        //self.state=State::Ready;
        self.state
    }

    #[inline(always)]
    pub fn load_registers(&mut self) {
        
    }

    pub fn save_registers(&mut self, ptr: u32) {
        let ptr = (ptr as *mut u32) as *mut [u32;14];
        let registers = unsafe{&(*ptr)};
        self.registers = registers.clone();
    }
}

//IDLE Thread
pub fn idle_thread() {
    println!("Idling..");
    /*
    loop{
    }
    */
    //TODO: need syscall EXIT
}

//TODO: Delete (war nur zu testzwecken^^)
pub fn idle_thread2() {
    println!("anderer thread");
    /*
    loop{
    }
    */
}
