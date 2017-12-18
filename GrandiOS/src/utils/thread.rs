use driver::serial::*;
use alloc::vec::Vec;
use alloc::string::String;
use utils::exceptions::common_code::RegisterStack;
use core::cmp::Ordering;
use swi::TCBStatistics;

#[derive(Debug)]
pub struct TCB {
    //TODO
    pub id: u32,
    pub name: String,
    // scheduling information
    pub cpu_time: u32,
    priority: u32,
    // ...
    pub register_stack: RegisterStack,
    //memory information (should later contain mmu parameters) for now it contains a memory location that is the used for the thread stack
    memory: Vec<u8>,
}

//TODO: sinnvollere ID Verwaltung / ID-Vergabe-Service (?)
static mut NEXT_ID: u32 = 0;

impl TCB {
    pub fn new(name: String, program_ptr: *const (), memory_size: usize, cpsr: u32) -> Self {
        let id;
        unsafe{
            id=NEXT_ID;
            NEXT_ID+=1;
        }
        println!("Created TCB with pc=\t{:p}",program_ptr);
        let memory = Vec::with_capacity(memory_size);
        let mut regs = RegisterStack::new();
        regs.lr_irq = program_ptr as u32; //regs[13] ist das LR und der PC wird aus dem LR geladen
        regs.sp = unsafe { memory.as_ptr().offset(memory_size as isize) as u32};
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

    pub fn load_registers(& self, registers: &mut RegisterStack) {
        registers.copy(& self.register_stack)
    }

    pub fn save_registers(&mut self, registers: & RegisterStack) {
        self.register_stack.copy(registers);
    }
    pub fn get_order(&self) -> u32 {
        self.id
    }
    pub fn get_statistics(&self) -> TCBStatistics {
        TCBStatistics {
            id: self.id,
            name: self.name.clone(),
            cpu_time: self.cpu_time,
            priority: self.priority,
            memory_allocated: (self.memory.capacity() as u32),
            memory_used: (self.memory.len() as u32),
        }
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
        self.id == other.id
    }
}
