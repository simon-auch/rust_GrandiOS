//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::TCB;
use alloc::vec_deque::VecDeque;
use utils::exceptions::software_interrupt;
use utils::registers;
use alloc::string::ToString;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    SCHEDULER = Some(Scheduler{
        running: Some(current_tcb),
        queue_ready: VecDeque::new(),
        queue_terminate: VecDeque::new(),
        queue_waiting_read: VecDeque::new(),
        queue_waiting_read_input: VecDeque::new(),
    });
    let scheduler = get_scheduler();
    let mut tcb_idle = TCB::new("Idle Thread".to_string(), idle as *mut _ , 0x100, registers::CPSR_MODE_USER);
    scheduler.add_thread(tcb_idle);
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
    running: Option<TCB>,
    //Queues for the threads
    queue_ready: VecDeque<TCB>,
    queue_terminate: VecDeque<TCB>,
    queue_waiting_read: VecDeque<TCB>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

impl Scheduler{
    pub fn add_thread(&mut self, tcb: TCB){
        self.queue_ready.push_back(tcb);
    }
    pub fn switch(&mut self, register_stack: &mut software_interrupt::RegisterStack, new_state: State){
        //save registers for current thread
        let mut running = self.running.take().unwrap();
        running.save_registers(&register_stack);
        //make sure the old thread gets the correct state and gets moved into the correct Queue
        match new_state{
            State::Ready       => { &mut self.queue_ready },
            State::Terminate   => { &mut self.queue_terminate },
            State::WaitingRead => {
                match self.queue_waiting_read_input.pop_front() {
                    None => { //there is no input available, so we need to wait
                        &mut self.queue_waiting_read
                    },
                    Some(c) => { //input is available, process it and make the thread ready
                        software_interrupt::work::read(&mut running, c);
                        &mut self.queue_ready
                    },
                }
            },
        }.push_back(running);
        let mut next = self.queue_ready.pop_front().unwrap();  //muss es immer geben, da idle thread
        next.load_registers(register_stack);
        self.running = Some(next);
    }

    //set of function to push something into the queue_waiting_*_input queues
    pub fn push_queue_waiting_read_input(&mut self, c: u8){
        match self.queue_waiting_read.pop_front() {
            None => { //No thread is waiting for this input
                self.queue_waiting_read_input.push_back(c);
            },
            Some(mut tcb) => {
                software_interrupt::work::read(&mut tcb, c);
                self.queue_ready.push_back(tcb);
            },
        }
    }
}


fn idle(){
    loop{}
}
