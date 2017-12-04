use driver::serial::*;
use alloc::vec::Vec;
use alloc::string::String;
use utils::exceptions::software_interrupt;
use core::cmp::Ordering;

#[derive(Debug)]
pub struct TCB {
    //TODO
    pub id: u32,
    pub name: String,
    // scheduling information
    pub cpu_time: u32,
    priority: u32,
    // ...
    pub register_stack: software_interrupt::RegisterStack,
    //memory information (should later contain mmu parameters) for now it contains a memory location that is the used for the thread stack
    memory: Vec<u8>,
}

//TODO: sinnvollere ID Verwaltung / ID-Vergabe-Service (?)
static mut NEXT_ID: u32 = 0;

impl TCB {
    pub fn new(name: String, program_ptr: *mut u32, memory_size: usize, cpsr: u32) -> Self {
        let id;
        unsafe{
            NEXT_ID+=1;
            id=NEXT_ID;
        }
        println!("Created TCB with pc=\t{:p}",program_ptr);
        let memory = Vec::with_capacity(memory_size);
        let mut regs : software_interrupt::RegisterStack = Default::default();
        regs.lr = program_ptr as u32; //regs[13] ist das LR und der PC wird aus dem LR geladen
        regs.sp = unsafe { memory.as_ptr().offset(memory_size as isize) as u32 };
        regs.cpsr = cpsr;
        TCB {
            id: id,
            name: name,
            cpu_time: 0,
            priority: 0,
            register_stack: regs,
            memory: memory,
        }
    }

    pub fn set_priority(&mut self, priority: u32){
        self.priority = priority;
    }
    pub fn get_priority(&self) -> u32 {
        self.priority
    }

    pub fn load_registers(&mut self, registers: &mut software_interrupt::RegisterStack) {
        registers.copy(&mut self.register_stack)
    }

    pub fn save_registers(&mut self, registers: & software_interrupt::RegisterStack) {
        self.register_stack.copy(registers);
    }
    pub fn get_order(&self) -> u32 {
        self.id
    }
}

//implements compare for the TCB, only uses the id field.
impl Eq for TCB {}
impl Ord for TCB {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_order().cmp(&other.get_order())
    }
}
impl PartialOrd for TCB {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for TCB {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}
