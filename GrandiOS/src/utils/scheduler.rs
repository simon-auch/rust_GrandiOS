//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::TCB;
use alloc::vec_deque::VecDeque;
use syscalls;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB, idle_thread: TCB){
    SCHEDULER = Some(Scheduler{
        running: current_tcb,
        queue_ready: VecDeque::new(),
        queue_terminate: VecDeque::new(),
        queue_waiting_read: VecDeque::new(),
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

//What states can our processes have?
pub enum State{
    Ready,
    Terminate,
    WaitingRead,
}

pub struct Scheduler{
    running: TCB,
    queue_ready: VecDeque<TCB>,
    queue_terminate: VecDeque<TCB>,
    queue_waiting_read: VecDeque<TCB>, //ist zur zeit der einzige blockierende syscall
}

impl Scheduler{
    pub fn get_current_tcb(&mut self) -> &mut TCB{
        &mut self.running
    }
    pub fn add_thread(&mut self, tcb: TCB){
        self.queue_ready.push_back(tcb);
    }
    pub fn switch(&mut self, register_stack: &mut syscalls::RegisterStack, new_state: State){
        let mut next_tcb = self.queue_ready.pop_front();
        match next_tcb {
            None => {
                //There is no other thread to which we could switch, so we dont switch.
                //The only moment where this should be possible to happen, is if the idle thread gets interrupted by an interrupt, but no other thread is ready.
                //This could be avoided if we would have 2 idle threads.
                //since we dont switch we can just ignore the new_state and the register_stack.
            },
            Some(mut next_tcb) => {
                self.running.save_registers(&register_stack);
                next_tcb.load_registers(register_stack);
                let mut old_running = replace(&mut self.running, next_tcb);
                //make sure the old thread gets the correct state and gets moved into the correct Queue
                match new_state{
                    State::Ready       => { &mut self.queue_ready },
                    State::Terminate   => { &mut self.queue_terminate },
                    State::WaitingRead => { &mut self.queue_waiting_read },
                }.push_back(old_running);
            }
        }
    }
}
