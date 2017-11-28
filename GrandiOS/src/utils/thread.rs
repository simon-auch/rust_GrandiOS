use driver::serial::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use syscalls;

pub struct TCB {
    //TODO
    pub id: u32,
    pub name: String,
    // scheduling information
    cpu_time: u32,
    // ...
    register_stack: syscalls::RegisterStack,
    //memory information (should later contain mmu parameters) for now it contains a memory location that is the used for the thread stack
    memory: Vec<u8>,
}

//TODO: sinnvollere ID Verwaltung / ID-Vergabe-Service (?)
static mut NEXT_ID: u32 = 0;

impl TCB {
    pub fn new(name: String, program_ptr: *mut u32, memory_size: usize) -> Self {
        let id;
        unsafe{
            NEXT_ID+=1;
            id=NEXT_ID;
        }
        println!("Created TCB with pc=\t{:p}",program_ptr);
        let memory = Vec::with_capacity(memory_size);
        let mut regs : syscalls::RegisterStack = Default::default();
        regs.lr = program_ptr as u32; //regs[13] ist das LR und der PC wird aus dem LR geladen
        regs.sp = unsafe { memory.as_ptr().offset(memory_size as isize) as u32 };
        TCB {
            id: id,
            name: name,
            cpu_time: 0,
            register_stack: regs,
            memory: memory,
        }
    }

    pub fn load_registers(&mut self, registers: &mut syscalls::RegisterStack) {
        registers.copy(&mut self.register_stack)
    }

    pub fn save_registers(&mut self, registers: & syscalls::RegisterStack) {
        self.register_stack.copy(registers);
    }
}

//IDLE Thread
pub fn idle_thread() {
    println!("Idling..");
    /*
    loop{
    }
    */
    //TODO: need syscall EXIT
}
