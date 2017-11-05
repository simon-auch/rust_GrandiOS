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
    pub id: u32,
    pub name: &'a str,
    // execution state
    state: State,
    // - registers
    //      R0-R16{_*}
    // - program counter,
    instr_counter: u32,
    //pub stack_pointer: u32,
    // scheduling information
    // ...
}

impl<'a> TCB<'a> {
    pub fn new(id: u32, name: &'a str) -> Self {
        TCB {
            id: id,
            name: name,
            state: State::Waiting,
            instr_counter: 0,
        }
    }

    pub fn get_state(&self) -> State {
        println!{"TODO: Remove me! @TCB.get_state()"};
        self.state
    }

    pub fn update_state(&mut self) -> State {
        println!{"TODO: Implement me! @TCB.update_state()"};
        self.state=State::Ready;
        self.state
    }

    pub fn load_registers(&self) {
        println!{"TODO: Implement me! @TCB.load_registers()"};
    }

    pub fn save_registers(&mut self) {
        println!{"TODO: Implement me! @TCB.save_registers()"};
    }
}
