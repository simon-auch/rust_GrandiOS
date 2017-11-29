//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::TCB;
use alloc::vec_deque::VecDeque;
use alloc::binary_heap::BinaryHeap;
use utils::exceptions::software_interrupt;
use utils::registers;
use alloc::string::ToString;
use core::cmp::Ordering;
use driver::serial::*;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    SCHEDULER = Some(Scheduler{
        running: Some(current_tcb),
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
    running: Option<TCB>,
    //Queues for the threads
    queue_ready: BinaryHeap<Priority<TCB>>,
    queue_terminate: BinaryHeap<Priority<TCB>>,
    queue_waiting_read: BinaryHeap<Priority<TCB>>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

fn add_thread_to_queue(queue: &mut BinaryHeap<Priority<TCB>>, tcb: TCB){
    queue.push(Priority{priority: tcb.get_priority(), data: tcb});
}

impl Scheduler{
    pub fn add_thread(&mut self, tcb: TCB){
        add_thread_to_queue(&mut self.queue_ready, tcb);
    }
    pub fn switch(&mut self, register_stack: &mut software_interrupt::RegisterStack, new_state: State){
        //save registers for current thread
        let mut running = self.running.take().unwrap();
        //println!("Switching from: {:?}", running);
        //println!("Current registers: {:?}", register_stack);
        running.save_registers(&register_stack);
        //make sure the old thread gets the correct state and gets moved into the correct Queue
        add_thread_to_queue(match new_state{
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
        }, running);
        let mut next = self.queue_ready.pop().unwrap().data;  //muss es immer geben, da idle thread
        //println!("Switching to: {:?}", next);
        next.load_registers(register_stack);
        self.running = Some(next);
    }

    //set of function to push something into the queue_waiting_*_input queues
    pub fn push_queue_waiting_read_input(&mut self, c: u8){
        match self.queue_waiting_read.pop() {
            None => { //No thread is waiting for this input
                self.queue_waiting_read_input.push_back(c);
            },
            Some(priority) => {
                let mut tcb = priority.data;
                software_interrupt::work::read(&mut tcb, c);
                add_thread_to_queue(&mut self.queue_ready, tcb);
            },
        }
    }
}


fn idle(){
    loop{}
}
