//Contains functions for the scheduler.

use core::mem::replace;
use utils::thread::TCB;
use alloc::heap::{Alloc, Layout};
use alloc::vec_deque::VecDeque;
use alloc::binary_heap::BinaryHeap;
use alloc::btree_map::BTreeMap;
use utils::exceptions::common_code::RegisterStack;
use utils::exceptions::software_interrupt;
use utils::registers;
use alloc::string::ToString;
use core::cmp::Ordering;
use driver::serial::*;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    let mut tcbs = BTreeMap::<u32, TCB>::new();
    let id = current_tcb.get_order();
    tcbs.insert(id, current_tcb);
    SCHEDULER = Some(Scheduler{
        tcbs: tcbs,
        running: id,
        queue_ready: BinaryHeap::new(),
        queue_terminate: BinaryHeap::new(),
        queue_waiting_read: BinaryHeap::new(),
        queue_waiting_read_input: VecDeque::new(),
    });
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
#[derive(Debug)]
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
    tcbs: BTreeMap<u32, TCB>,
    running: u32,
    //Queues for the threads
    queue_ready: BinaryHeap<Priority<u32>>,
    queue_terminate: BinaryHeap<Priority<u32>>,
    queue_waiting_read: BinaryHeap<Priority<u32>>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

fn add_thread_to_queue(queue: &mut BinaryHeap<Priority<u32>>, tcb: & TCB){
    queue.push(Priority{priority: tcb.get_priority(), data: tcb.get_order()});
}

impl Scheduler{
    pub fn add_thread(&mut self, tcb: TCB){
        add_thread_to_queue(&mut self.queue_ready, & tcb);
        let id = tcb.get_order();
        self.tcbs.insert(id, tcb);
    }
    pub fn switch(&mut self, register_stack: &mut RegisterStack, new_state: State){
        {//save registers for current thread and move it into the correct queue
        let mut running = self.tcbs.get_mut(&self.running).unwrap();
        //println!("Switching from: {}", running.name);
        //println!("Current registers: {:#?}", register_stack);
        running.save_registers(&register_stack);
        //make sure the old thread gets the correct state and gets moved into the correct Queue
        add_thread_to_queue(match new_state{
            State::Ready       => {
                //println!("queue_ready");
                &mut self.queue_ready
            },
            State::Terminate   => { 
                //println!("queue_terminate"); 
                &mut self.queue_terminate
            },
            State::WaitingRead => {
                match self.queue_waiting_read_input.pop_front() {
                    None => { //there is no input available, so we need to wait
                        //println!("queue_waiting_read");
                        &mut self.queue_waiting_read
                    },
                    Some(c) => { //input is available, process it and make the thread ready
                        //println!("queue_waiting_read -> queue_ready");
                        software_interrupt::work::read(&mut running, c);
                        &mut self.queue_ready
                    },
                }
            },
        }, & running);
        }
        //println!("queue_ready: {:#?}", self.queue_ready);
        let mut next = self.tcbs.get_mut(&(self.queue_ready.pop().unwrap().data)).unwrap();  //muss es immer geben, da idle thread
        //println!("Switching to: {}", next.name);
        //println!("Loading Registers");
        (&next).load_registers(register_stack);
        //println!("Current registers: {:#?}", register_stack);
        self.running = next.get_order();
    }

    //set of function to push something into the queue_waiting_*_input queues
    pub fn push_queue_waiting_read_input(&mut self, c: u8){
        match self.queue_waiting_read.pop() {
            None => { //No thread is waiting for this input
                self.queue_waiting_read_input.push_back(c);
            },
            Some(priority) => {
                let id = priority.data;
                let mut tcb = self.tcbs.get_mut(&id).unwrap();
                software_interrupt::work::read(&mut tcb, c);
                add_thread_to_queue(&mut self.queue_ready, & tcb);
            },
        }
    }
    pub fn alloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut running = self.tcbs.get_mut(&self.running).unwrap();
        running.allocs.push((ptr, layout));
    }
    pub fn exit(&mut self) {
        let r = self.running;
        //TODO: send parent result of thread
        self.kill(r);
        self.running = 0; //switch to idle thread to not care about registers
    }
    pub fn kill(&mut self, id: u32) {
        if id == 0 { return; } //We do NOT kill the idle thread!
        if !self.tcbs.contains_key(&id) { return; }
        let mut tcb = self.tcbs.remove(&id).unwrap();
        for (ptr, layout) in tcb.allocs.into_iter() {
            unsafe {
                (&mut &::GLOBAL).dealloc(ptr, layout);
            }
        }
    }
}


pub extern fn idle(){
    loop{}
}
