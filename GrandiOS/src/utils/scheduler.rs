//Contains functions for the scheduler.

use utils::thread::TCB;
use swi::TCBStatistics;
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;
use alloc::binary_heap::BinaryHeap;
use alloc::btree_map::BTreeMap;
use utils::exceptions::common_code::RegisterStack;
use utils::exceptions::software_interrupt;
use core::cmp::Ordering;
use core::u16;
//use driver::serial::*;
use driver::system_timer::*;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    let mut tcbs = BTreeMap::<TCB_ID, TCB>::new();
    let id = TCB_ID(current_tcb.get_order());
    tcbs.insert(id, current_tcb);
    SCHEDULER = Some(Scheduler{
        tcbs: tcbs,
        running: id,
        queue_ready: BinaryHeap::new(),
        queue_terminate: BinaryHeap::new(),
        queue_waiting_read: BinaryHeap::new(),
        queue_waiting_read_input: VecDeque::new(),
        queue_waiting_sleep: BinaryHeap::new(),
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
    Sleep,
}

//struct that contains a something and a priority which to use for scheduling
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

//Wraper to revrse the order of something
#[derive(PartialEq)]
struct ReverseOrder<T: Ord> {
    data: T,
}
impl<T: Ord> Eq for ReverseOrder<T> {}
impl<T: Ord> PartialOrd for ReverseOrder<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.data.partial_cmp(&self.data)
    }
}
impl<T: Ord> Ord for ReverseOrder<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

//Macro to create an identifier, which can be compared, but not used in math.
macro_rules! identifier (
    ($name:ident) => {
        #[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]
        struct TCB_ID(u32);
    }
);

identifier!(TCB_ID);

pub struct Scheduler{
    tcbs: BTreeMap<TCB_ID, TCB>,
    running: TCB_ID,
    //Queues for the threads
    queue_ready: BinaryHeap<Priority<TCB_ID>>,
    queue_terminate: BinaryHeap<Priority<TCB_ID>>,
    queue_waiting_read: BinaryHeap<Priority<TCB_ID>>,
    queue_waiting_sleep: BinaryHeap<ReverseOrder<Priority<TCB_ID>>>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

fn add_thread_to_queue(queue: &mut BinaryHeap<Priority<TCB_ID>>, tcb: & TCB){
    queue.push(Priority{priority: tcb.get_priority(), data: TCB_ID(tcb.get_order())});
}

impl Scheduler{
    pub fn add_thread(&mut self, tcb: TCB){
        add_thread_to_queue(&mut self.queue_ready, & tcb);
        let id = TCB_ID(tcb.get_order());
        self.tcbs.insert(id, tcb);
    }
    pub fn switch(&mut self, register_stack: &mut RegisterStack, new_state: State){
        let mut st =  unsafe{ get_system_timer() };
        let mut current_time = st.get_current_ticks();//returns ticks, default to roughly 1ms
        let mut next_wanted_wakeup : u16 = 0xFFFF; //if the st.piv is set to 0xFFFF the next system timer interrupt happens in (a bit less then) 2seconds
        {//save registers for current thread and move it into the correct queue
        let mut running = self.tcbs.get_mut(&self.running).unwrap();
        //println!("Switching from: {}", running.name);
        //println!("Current registers: {:#?}", register_stack);
        running.save_registers(&register_stack);
        //make sure the old thread gets the correct state and gets moved into the correct Queue
        match new_state{
            State::Ready       => {
                //println!("queue_ready");
                add_thread_to_queue(&mut self.queue_ready, & running);
            },
            State::Terminate   => { 
                //println!("queue_terminate"); 
                add_thread_to_queue(&mut self.queue_terminate, & running);
            },
            State::WaitingRead => {
                match self.queue_waiting_read_input.pop_front() {
                    None => { //there is no input available, so we need to wait
                        //println!("queue_waiting_read");
                        add_thread_to_queue(&mut self.queue_waiting_read, & running);
                    },
                    Some(c) => { //input is available, process it and make the thread ready
                        //println!("queue_waiting_read -> queue_ready");
                        software_interrupt::work::read(&mut running, c);
                        add_thread_to_queue(&mut self.queue_ready, & running);
                    },
                }
            },
            State::Sleep => {
                //add thread to sleeping queue, with priority set to the time it wants o sleep
                let t = software_interrupt::work::sleep(&mut running);
                self.queue_waiting_sleep.push(ReverseOrder{data: Priority{priority: current_time+t, data: TCB_ID(running.get_order())}});
            }
        }
        }
        {//free resources of threads in the terminate queue (this could also only be done when switching to idle thread or so, but that would be an perf. improvement)
        while let Some(priority) = self.queue_terminate.pop() {
            self.terminate(priority.data);
        }
        while let Some(rev_ord) = self.queue_waiting_sleep.pop() {
            let mut priority = rev_ord.data;
            if priority.priority <= current_time {
                let id  = priority.data;
                let tcb = self.tcbs.get(&id).unwrap();
                add_thread_to_queue(&mut self.queue_ready, & tcb);
            } else {
                let delta_ticks = priority.priority - current_time;
                let delta_ticks : u16 = if delta_ticks > u16::MAX as u32 { 0xFFFF } else { delta_ticks as u16 };
                let next_wanted_wakeup_temp = st.ticks_to_piv(delta_ticks);
                if next_wanted_wakeup_temp < next_wanted_wakeup {
                    //next_wanted_wakeup_temp cant be zero, because that would imply that delta_ticks was zero, but then we would have taken the other branch of the if.
                    next_wanted_wakeup = next_wanted_wakeup_temp;
                }
                self.queue_waiting_sleep.push(ReverseOrder{data: priority});
                break;
            }
        }
        }
        //println!("queue_ready: {:#?}", self.queue_ready);
        let mut next = self.tcbs.get_mut(&(self.queue_ready.pop().unwrap().data)).unwrap();  //muss es immer geben, da idle thread
        //println!("Switching to: {}", next.name);
        //println!("Loading Registers");
        (&next).load_registers(register_stack);
        //println!("Current registers: {:#?}", register_stack);
        self.running = TCB_ID(next.get_order());
        
        //configure the st to wake us up when we need it.
        //println!("Setting piv to: 0x{:x}", next_wanted_wakeup);
        st.set_piv(next_wanted_wakeup);
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
    fn terminate(&mut self, id: TCB_ID) {
        if id == TCB_ID(0) { return; } //We do NOT kill the idle thread!
        match self.tcbs.remove(&id){
            None => { //No thread with the given id exists 
            },
            Some(tcb) => {
                //Cleanup code
            },
        }
    }
    pub extern fn get_all_tcb_statistics(&self) -> Vec<TCBStatistics> {
        let values: Vec<&TCB> = self.tcbs.values().collect();
        let mut statistics = Vec::with_capacity(values.len());
        for tcb in values {
            statistics.push(tcb.get_statistics());
        }
        statistics
    }
}


pub extern fn idle(){
    loop{
        //This asm makes the cpu idle until an interrupt occurs. see ARM920T_TRM1_S.pdf page 42 (2-18)
        unsafe{asm!("
            mcr p15,0,r0,c7,c0,4"
            :::"memory","r0":"volatile"
        );}
    }
}
