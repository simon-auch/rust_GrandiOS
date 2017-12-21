//Contains functions for the scheduler.

use utils::thread::TCB;
use swi;
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;
use alloc::binary_heap::BinaryHeap;
use alloc::btree_map::BTreeMap;
use alloc::btree_set::BTreeSet;
use utils::exceptions::common_code::RegisterStack;
use utils::exceptions::software_interrupt;
use core::cmp::Ordering;
use core::u16;
use core::iter::FromIterator;
use driver::serial::*;
use driver::system_timer::*;

static mut SCHEDULER: Option<Scheduler> = None;

pub unsafe fn init(current_tcb: TCB){
    let mut tcbs = BTreeMap::<TCB_ID, TCB>::new();
    let id = TCB_ID(current_tcb.get_order());
    tcbs.insert(id, current_tcb);
    SCHEDULER = Some(Scheduler{
        tcbs: tcbs,
        running: id,
        select_counter: 0,
        selects: BTreeMap::new(),
        queue_ready: BinaryHeap::new(),
        queue_terminate: BinaryHeap::new(),
        queue_waiting_read: BTreeSet::new(),
        queue_waiting_read_input: VecDeque::new(),
        queue_waiting_sleep: BTreeSet::new(),
    });
}

pub unsafe fn get_scheduler() -> &'static mut Scheduler {
    match SCHEDULER {
        Some(ref mut sched) => &mut *sched,
        None => panic!(),
    }
}

//What states can our processes have?
#[derive(Debug)]
pub enum State{
    Ready,
    Terminate,
    Waiting(Vec<u32>), //swi numbers
}

//struct that contains a something and a priority which to use for scheduling
#[derive(Debug, Clone)]
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
#[derive(Debug)]
#[derive(PartialEq, Clone)]
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
        #[allow(non_camel_case_types)]
        #[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Copy)]
        struct $name(u32);
    }
);

identifier!(TCB_ID);
identifier!(SELECT_ID);

#[derive(Debug)]
struct Select {
    tcb_id: TCB_ID,
}

#[derive(Debug)]
pub struct Scheduler{
    tcbs: BTreeMap<TCB_ID, TCB>,
    running: TCB_ID,
    select_counter: u32,
    selects: BTreeMap<SELECT_ID, Select>,
    //Queues for the threads
    queue_ready: BinaryHeap<Priority<TCB_ID>>,
    queue_terminate: BinaryHeap<Priority<TCB_ID>>,
    queue_waiting_read: BTreeSet<Priority<SELECT_ID>>,
    queue_waiting_sleep: BTreeSet<ReverseOrder<Priority<SELECT_ID>>>,
    //Queues for stuff that threads can wait for
    queue_waiting_read_input: VecDeque<u8>,
}

/*
fn add_thread_to_queue(queue: &mut BinaryHeap<Priority<TCB_ID>>, tcb: & TCB){
    queue.push(Priority{priority: tcb.get_priority(), data: TCB_ID(tcb.get_order())});
}
*/

fn btree_set_min<U: Clone + Ord>(set: &mut BTreeSet<U>) -> Option<U> {
    let mut ret: Option<U>;
    {
    let iterator = set.iter();
    match set.iter().next() {
        None => {
            ret = None;
        },
        Some(u) => {
            ret = Some(u.clone());
        }
    }
    }
    match ret {
        None => {
            return None;
        },
        Some(u) => {
            set.remove(&u);
            return Some(u);
        }
    }
}

impl Scheduler{
    fn print(&self){
        println!("{:#?}", &self);
    }
    pub fn add_thread(&mut self, tcb: TCB){
	let id = TCB_ID(tcb.get_order());
        self.queue_ready.push(Priority{priority: tcb.get_priority(), data: id});
        self.tcbs.insert(id, tcb);
    }
    fn remove_select(&mut self, select_id: & SELECT_ID){
        let mut queue_filtered = BTreeSet::from_iter(self.queue_waiting_read.iter().filter(|prio| & prio.data != select_id).map(|v| v.clone()));
        self.queue_waiting_read = queue_filtered;
        let mut queue_filtered = BTreeSet::from_iter(self.queue_waiting_sleep.iter().filter(|prio| & prio.data.data != select_id).map(|v| v.clone()));
        self.queue_waiting_sleep = queue_filtered;
        self.selects.remove(select_id);
    }
    //part of switch which safes the state of the running process
    fn switch_running_thread(&mut self, register_stack: &mut RegisterStack, new_state: State, current_time: u32, next_wanted_wakeup : &mut u16){
        let mut running_id = self.running;
        let mut running = self.tcbs.get_mut(&running_id).unwrap();
        //println!("Switching from: {} -> {:?}", running.name, new_state);
        //println!("Current registers: {:#?}", register_stack);
        running.save_registers(&register_stack);
        //make sure the old thread gets the correct state and gets moved into the correct Queue
        match new_state{
            State::Ready       => {
                //println!("queue_ready");
                self.queue_ready.push(Priority{priority: running.get_priority(), data: running_id});
            },
            State::Terminate   => {
                //println!("queue_terminate");
                self.queue_terminate.push(Priority{priority: running.get_priority(), data: running_id});
            },
            State::Waiting(swis) => {
                let mut count: u32 = 0;
                let select = Select{tcb_id: running_id};
                let select_id = SELECT_ID(self.select_counter);
                for swi_number in swis {
                    match swi_number {
                        READ!() => {
                            count += 1;
                            self.queue_waiting_read.insert(Priority{priority: 0, data: select_id});
                        },
                        SLEEP!() => {
                            count += 1;
                            let t = software_interrupt::work::sleep_get_ticks(&mut running);
                            self.queue_waiting_sleep.insert(ReverseOrder{data: Priority{priority: current_time+t, data: select_id}});
                        },
                        _ => {},
                    }
                }
                if count > 0 {
                    self.selects.insert(select_id, select);
                    self.select_counter += 1;
                }
            },
        }
    }
    pub fn switch(&mut self, register_stack: &mut RegisterStack, new_state: State){
        println!("");
        //self.print();

        let mut st =  unsafe{ get_system_timer() };
        let mut current_time = st.get_current_ticks();//returns ticks, default to roughly 1ms
        let mut next_wanted_wakeup : u16 = 0xFFFF; //if the st.piv is set to 0xFFFF the next system timer interrupt happens in (a bit less then) 2seconds
        self.switch_running_thread(register_stack, new_state, current_time, &mut next_wanted_wakeup);
        {//free resources of threads in the terminate queue (this could also only be done when switching to idle thread or so, but that would be an perf. improvement)
        while let Some(priority) = self.queue_terminate.pop() {
            self.terminate(priority.data);
        }
        while let Some(rev_ord) = btree_set_min(&mut self.queue_waiting_sleep) {
            let mut priority = rev_ord.data;
            if priority.priority <= current_time {
                let select_id  = priority.data;
                {
                let select = self.selects.get(& select_id).unwrap();
                let tcb_id = select.tcb_id;
                let mut tcb = self.tcbs.get_mut(&tcb_id).unwrap();
                software_interrupt::work::sleep(&mut tcb);
                self.queue_ready.push(Priority{priority: tcb.get_priority(), data: tcb_id});
                }
                self.remove_select(& select_id);
            } else {
                let delta_ticks = priority.priority - current_time;
                let delta_ticks : u16 = if delta_ticks > u16::MAX as u32 { 0xFFFF } else { delta_ticks as u16 };
                let next_wanted_wakeup_temp = st.ticks_to_piv(delta_ticks);
                if next_wanted_wakeup_temp < next_wanted_wakeup {
                    //next_wanted_wakeup_temp cant be zero, because that would imply that delta_ticks was zero, but then we would have taken the other branch of the if.
                    next_wanted_wakeup = next_wanted_wakeup_temp;
                }
                self.queue_waiting_sleep.insert(ReverseOrder{data: priority});
                break;
            }
        }
        while (self.queue_waiting_read.len() > 0) && (self.queue_waiting_read_input.len() > 0) {
            let c = self.queue_waiting_read_input.pop_front().unwrap();
            let select_id = btree_set_min(&mut self.queue_waiting_read).unwrap().data;
            {
            let select = self.selects.get(& select_id).unwrap();
            let tcb_id = select.tcb_id;
            let mut tcb = self.tcbs.get_mut(&tcb_id).unwrap();
            software_interrupt::work::read(&mut tcb, c);
            self.queue_ready.push(Priority{priority: tcb.get_priority(), data: tcb_id});
            }
            self.remove_select(& select_id);
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
        //make sure the next wanted wakeup is not too close
        let next_wanted_wakeup_temp = st.ticks_to_piv(10); //minimum of 10 ticks per timeslice
        if next_wanted_wakeup_temp > next_wanted_wakeup {
            next_wanted_wakeup = next_wanted_wakeup_temp;
        }
        st.set_piv(next_wanted_wakeup);
    }

    //set of function to push something into the queue_waiting_*_input queues
    pub fn push_queue_waiting_read_input(&mut self, c: u8){
        self.queue_waiting_read_input.push_back(c);
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
    pub extern fn get_all_tcb_statistics(&self) -> Vec<swi::TCBStatistics> {
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
