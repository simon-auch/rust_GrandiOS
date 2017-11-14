//Dokumentation: S.239

use core::ptr::{write_volatile, read_volatile};

pub const IT_BASE_ADDRESS : u32 = 0x0;
pub const AIC_BASE_ADDRESS : u32 = 0xFFFFF000;

//lots of consts for all the register bits
const ICR_FIQ: u32 = 1<<0;
const ICR_SYS: u32 = 1<<1;
const ICR_PID: [u32; 30] = [1<<2, 1<<3, 1<<4, 1<<5, 1<<6, 1<<7, 1<<8, 1<<9, 1<<10, 1<<11, 1<<12, 1<<13, 1<<14, 1<<15, 1<<16, 1<<17, 1<<18, 1<<19, 1<<20, 1<<21, 1<<22, 1<<23, 1<<24, 1<<25, 1<<26, 1<<27, 1<<28, 1<<29, 1<<30, 1<<31]; 

#[repr(C)]
struct InterruptTableMemoryMap{
	reset: u32,
	undefined_instruction: u32,
	software_interrupt: u32,
	prefetch_abort: u32,
	data_abort: u32,
	reserved_0: u32, //see: http://osnet.cs.nchu.edu.tw/powpoint/Embedded94_1/Chapter%207%20ARM%20Exceptions.pdf
	irq: u32,
	fiq: u32,
}

#[repr(C)]
struct AdvancedInterruptControllerMemoryMap{
	smr: [u32; 32],		//source mode register 0-31
	svr: [u32; 32],		//source vector register 0-31
	ivr: u32,			//interrupt vector register
	fivr: u32,			//fast interrupt vector reigster
	isr: u32,			//interrupt status register
	ipr: u32,			//interrupt pending register
	imr: u32,			//interrupt mask register
	cisr: u32,			//core interrupt status register
	reserved_0: [u32; 2],
	iecr: u32,			//interrupt enable command register
	idcr: u32,			//interrupt disable command register
	iccr: u32,			//interrupt clear command register
	iscr: u32,			//interrupt set command register
	eoicr: u32,			//end of interrupt command register
	sivr: u32,			//spurious interrupt vector register
	dcr: u32,			//debug control register
	reserved_1: [u32; 1],
}

pub struct InterruptController{
	itmm: *mut InterruptTableMemoryMap,
	aicmm: *mut AdvancedInterruptControllerMemoryMap,
}

unsafe impl Send for InterruptController { }

impl InterruptController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(it_base_address: u32, aic_base_address: u32) -> Self{
        let mut ic = InterruptController{
			itmm: it_base_address as *mut InterruptTableMemoryMap,
            aicmm: aic_base_address as *mut AdvancedInterruptControllerMemoryMap,
        };
        ic.init();
        return ic;
    }
    fn init(&mut self){
		let ldr = 0xe51fff20; //ldr pc,[pc,#-0xF20]
		unsafe{
			write_volatile(&mut (*(self.itmm)).irq, ldr);
		}
	}
    pub fn enable(&mut self){
		unsafe{
		    write_volatile(&mut (*(self.aicmm)).iecr, ICR_SYS);
        }
	}
	pub fn disable(&mut self){
		unsafe{
		    write_volatile(&mut (*(self.aicmm)).idcr, ICR_SYS);
        }
	}
	pub fn set_handler(&mut self, interrupt_line: usize, f: extern fn()){
		assert!(interrupt_line < 32, "interrupt line must be between 0 and 31");
		unsafe{
		    write_volatile(&mut (*(self.aicmm)).svr[interrupt_line], f as u32);
        }
	}
    pub fn set_priority(&mut self, interrupt_line: usize, priority: u32){
        assert!(interrupt_line < 32, "interrupt line must be between 0 and 31");
        assert!(priority < 8, "priority must be between 0 and 7");
        let mut reg : u32 = unsafe{ read_volatile(&mut (*(self.aicmm)).smr[interrupt_line])};
        //clear old priority
        reg &= (0xFFFFFFFF ^ 0b111);
        reg |= priority;
        unsafe{
            write_volatile(&mut (*(self.aicmm)).smr[interrupt_line], reg);
        }
    }
    pub fn set_sourcetype(&mut self, interrupt_line: usize, sourcetype: u32){
        assert!(interrupt_line < 32, "interrupt line must be between 0 and 31");
        assert!(sourcetype < 4, "priority must be between 0 and 3");
        let mut reg : u32 = unsafe{ read_volatile(&mut (*(self.aicmm)).smr[interrupt_line])};
        //clear old sourcetype
        reg &= (0xFFFFFFFF ^ (0b11 << 5));
        reg |= sourcetype << 5;
        unsafe{
            write_volatile(&mut (*(self.aicmm)).smr[interrupt_line], reg);
        }
    }
    pub fn end_interrupt(&mut self){
        unsafe{
            write_volatile(&mut (*(self.aicmm)).eoicr, 0);
        }
    }
}
