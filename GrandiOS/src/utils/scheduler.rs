//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::{TCB, State};
use alloc::vec_deque::VecDeque;
use syscalls;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB, idle_thread: TCB){
    SCHEDULER = Some(Scheduler{
        current_tcb: current_tcb,
        tcbs: VecDeque::new(),
    });
    let scheduler = get_scheduler();
    scheduler.add_thread(idle_thread);
}

pub unsafe fn get_scheduler() -> &'static mut Scheduler {
    match SCHEDULER {
        Some(ref mut sched) => &mut *sched,
        None => panic!(),
    }
}

pub struct Scheduler{
    current_tcb: TCB,
    tcbs: VecDeque<TCB>,
}

impl Scheduler{
    pub fn get_current_tcb(&mut self) -> &mut TCB{
        &mut self.current_tcb
    }
    pub fn add_thread(&mut self, tcb: TCB){
        self.tcbs.push_back(tcb);
    }
    pub fn switch(&mut self, register_stack: &mut syscalls::RegisterStack){
        self.current_tcb.save_registers(&register_stack);
        let mut next_tcb = self.tcbs.pop_front().unwrap();
        loop {
            match next_tcb.state{
                State::Ready => {break;},
                _ => {
                    self.tcbs.push_back(next_tcb);
                    next_tcb = self.tcbs.pop_front().unwrap();
                },
            }
        }
        next_tcb.load_registers(register_stack);
        let current_tcb = replace(&mut self.current_tcb, next_tcb);
        self.tcbs.push_back(current_tcb);
    }
}
