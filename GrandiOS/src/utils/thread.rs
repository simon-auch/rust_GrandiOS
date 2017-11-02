extern crate alloc;
use alloc::boxed::Box;


#[derive(Copy,Clone)]
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
    // - program counter,
    instr_counter: u32,
    // - stack pointer
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
        self.state
    }

    pub fn update_state(&mut self) {
        // TODO: Implement me!"
        self.state=State::Ready;
    }
}

impl alloc::fmt::Display for State {
    fn fmt(&self, f: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
        let state_name = match *self {
            State::Running => "Running",
            State::Ready => "Ready",
            State::Waiting => "Waiting",
            State::Terminated => "Terminated",
        };
        write!(f, "{}", state_name)
    }
}
