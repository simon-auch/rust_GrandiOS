//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::TCB;
use alloc::vec_deque::VecDeque;
use alloc::binary_heap::BinaryHeap;
use utils::exceptions::software_interrupt;
use utils::registers;
use alloc::string::ToString;
use core::cmp::Ordering;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    SCHEDULER = Some(Scheduler{
        running: current_tcb,
        queue_ready: BinaryHeap::new(),
        queue_terminate: BinaryHeap::new(),
        queue_waiting_read: BinaryHeap::new(),
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

//struct that contains a tcb and a priority which to use for scheduling
struct Priority<T>{
    priority: u32,
    data: T,
}
//impls we need for BinaryHeap to work
impl<T> Eq for Priority<T> {}
impl<T> Ord for Priority<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}
impl<T> PartialOrd for Priority<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> PartialEq for Priority<T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

pub struct Scheduler{
    running: TCB,
    //Queues for the threads
    queue_ready: BinaryHeap<Priority<TCB>>,
    queue_terminate: BinaryHeap<Priority<TCB>>,
    queue_waiting_read: BinaryHeap<Priority<TCB>>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

impl Scheduler{
    pub fn get_current_tcb(&mut self) -> &mut TCB{
        &mut self.running
    }
    pub fn add_thread(&mut self, tcb: TCB){
        self.queue_ready.push(tcb);
    }
    pub fn switch(&mut self, register_stack: &mut software_interrupt::RegisterStack, new_state: State){
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
                    State::WaitingRead => { 
                        match self.queue_waiting_read_input.pop_front() {
                            None => { //there is no input available, so we need to wait
                                &mut self.queue_waiting_read
                            },
                            Some(c) => { //input is available, process it and make the thread ready
                                software_interrupt::work::read(&mut old_running, c);
                                &mut self.queue_ready
                            },
                        }
                    },
                }.push(Priority{priority: old_running.get_priority(), data: old_running});
            }
        }
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
